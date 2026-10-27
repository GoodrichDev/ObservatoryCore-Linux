//! Bounded off-thread HTTP for installed native plugins.
use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderName, HeaderValue},
    Method, Url,
};
use serde_json::{json, Map, Value};
use std::{
    io::Read,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender, TrySendError},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

const WORKERS: usize = 4;
const QUEUE: usize = 64;
const MAX_REQUEST: usize = 256 * 1024;
const MAX_BODY: usize = 4 * 1024 * 1024;
const MAX_RESPONSE_HEADERS: usize = 64 * 1024;

pub struct HttpResponse {
    pub plugin_id: String,
    pub data: Value,
}

struct Request {
    plugin_id: String,
    id: String,
    url: Url,
    method: Method,
    headers: HeaderMap,
    body: Option<String>,
    include_headers: bool,
    submitted: Instant,
    deadline: Instant,
}

pub struct HttpClient {
    requests: Option<SyncSender<Request>>,
    responses: Receiver<HttpResponse>,
    cancelled: Arc<AtomicBool>,
    startup_error: Option<String>,
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .connect_timeout(Duration::from_secs(10))
            .redirect(reqwest::redirect::Policy::limited(10))
            .user_agent(concat!("ObservatoryCore/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|_| "Cannot initialize the native HTTP client".to_owned());
        Self::with_client(client)
    }

    fn with_client(client: Result<Client, String>) -> Self {
        let startup_error = client.as_ref().err().cloned();
        let (request_tx, request_rx) = mpsc::sync_channel::<Request>(QUEUE);
        let requests = Arc::new(Mutex::new(request_rx));
        let (response_tx, response_rx) = mpsc::sync_channel::<HttpResponse>(QUEUE);
        let cancelled = Arc::new(AtomicBool::new(false));
        if let Ok(client) = client {
            for worker in 0..WORKERS {
                let input = requests.clone();
                let output = response_tx.clone();
                let client = client.clone();
                let token = cancelled.clone();
                let spawned = thread::Builder::new()
                    .name(format!("observatory-http-{worker}"))
                    .spawn(move || loop {
                        let request = match input.lock() {
                            Ok(input) => match input.recv() {
                                Ok(request) => request,
                                Err(_) => break,
                            },
                            Err(_) => break,
                        };
                        if token.load(Ordering::Relaxed) {
                            break;
                        }
                        let response = execute(&client, request);
                        if token.load(Ordering::Relaxed) || output.send(response).is_err() {
                            break;
                        }
                    });
                if spawned.is_err() {
                    cancelled.store(true, Ordering::Relaxed);
                    return Self {
                        requests: None,
                        responses: response_rx,
                        cancelled,
                        startup_error: Some("Cannot start the native HTTP worker pool".into()),
                    };
                }
            }
        }
        Self {
            requests: Some(request_tx),
            responses: response_rx,
            cancelled,
            startup_error,
        }
    }

    pub fn submit(&self, plugin_id: String, request: Value) -> Result<(), String> {
        if let Some(error) = &self.startup_error {
            return Err(error.clone());
        }
        if plugin_id.is_empty() || plugin_id.len() > 512 || plugin_id.chars().any(char::is_control)
        {
            return Err("Invalid originating HTTP plugin ID".into());
        }
        let request = parse_request(plugin_id, &request)?;
        self.requests
            .as_ref()
            .ok_or("HTTP service is closed")?
            .try_send(request)
            .map_err(|error| match error {
                TrySendError::Full(_) => "HTTP request queue is full".into(),
                TrySendError::Disconnected(_) => "HTTP service is closed".into(),
            })
    }

    pub fn poll(&self) -> Vec<HttpResponse> {
        self.responses.try_iter().collect()
    }
}

impl Drop for HttpClient {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Relaxed);
        // Running requests keep their <=10s deadline; pending work is discarded.
        // Do not join network workers on the ingestion/UI path.
        self.requests.take();
    }
}

/// Lightweight validation for transactional Lua staging. No network I/O occurs.
pub fn validate_request(request: &Value) -> Result<(), String> {
    parse_request(String::new(), request).map(|_| ())
}

fn parse_request(plugin_id: String, value: &Value) -> Result<Request, String> {
    if !value.is_object() {
        return Err("HTTP request must be an object".into());
    }
    if serde_json::to_vec(value)
        .map_err(|_| "Invalid HTTP request JSON")?
        .len()
        > MAX_REQUEST
    {
        return Err("HTTP request exceeds 256 KiB".into());
    }
    let id = value["request_id"]
        .as_str()
        .filter(|id| !id.is_empty() && id.len() <= 512 && !id.chars().any(char::is_control))
        .ok_or("HTTP request_id must be a nonempty string of at most 512 bytes")?
        .to_owned();
    let url = value["url"]
        .as_str()
        .filter(|url| url.len() <= 8192)
        .ok_or("HTTP request requires a URL of at most 8192 bytes")?;
    let url = Url::parse(url).map_err(|_| "Invalid HTTP request URL")?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err("HTTP requests support only http and https URLs".into());
    }
    let method = match value.get("method") {
        None => Method::GET,
        Some(Value::String(method)) if method.len() <= 32 => {
            Method::from_bytes(method.as_bytes()).map_err(|_| "Invalid HTTP method")?
        }
        _ => return Err("HTTP method must be a string of at most 32 bytes".into()),
    };
    let body = match value.get("body") {
        None | Some(Value::Null) => None,
        Some(Value::String(body)) => Some(body.clone()),
        _ => return Err("HTTP body must be a string".into()),
    };
    let mut headers = HeaderMap::new();
    if let Some(value) = value.get("headers") {
        let source = value.as_object().ok_or("HTTP headers must be an object")?;
        if source.len() > 128 {
            return Err("HTTP request has too many headers".into());
        }
        for (name, values) in source {
            let name =
                HeaderName::from_bytes(name.as_bytes()).map_err(|_| "Invalid HTTP header name")?;
            let values = if let Some(value) = values.as_str() {
                vec![value]
            } else if let Some(values) = values.as_array() {
                values
                    .iter()
                    .map(|value| value.as_str().ok_or("HTTP header values must be strings"))
                    .collect::<Result<Vec<_>, _>>()?
            } else {
                return Err("HTTP header values must be strings or string arrays".into());
            };
            if headers.len() + values.len() > 128 {
                return Err("HTTP request has too many header values".into());
            }
            for value in values {
                headers.append(
                    name.clone(),
                    HeaderValue::from_str(value).map_err(|_| "Invalid HTTP header value")?,
                );
            }
        }
    }
    let timeout = match value.get("timeout_ms") {
        None => 10_000,
        Some(value) => value
            .as_u64()
            .filter(|value| *value > 0)
            .ok_or("HTTP timeout_ms must be a positive integer")?
            .min(10_000),
    };
    let include_headers = match value.get("include_headers") {
        None => false,
        Some(Value::Bool(value)) => *value,
        _ => return Err("HTTP include_headers must be a boolean".into()),
    };
    let submitted = Instant::now();
    Ok(Request {
        plugin_id,
        id,
        url,
        method,
        headers,
        body,
        include_headers,
        submitted,
        deadline: submitted + Duration::from_millis(timeout),
    })
}

fn execute(client: &Client, request: Request) -> HttpResponse {
    let mut data = json!({"request_id":request.id,"ok":false,"status":null,"body":null,"body_base64":null,
        "error":null,"error_kind":null,"truncated":false});
    let result = (|| -> Result<(), (&'static str, &'static str)> {
        let remaining = request
            .deadline
            .checked_duration_since(Instant::now())
            .filter(|value| !value.is_zero())
            .ok_or(("timeout", "HTTP request timed out while queued"))?;
        let mut builder = client
            .request(request.method, request.url)
            .headers(request.headers)
            .timeout(remaining);
        if let Some(body) = request.body {
            builder = builder.body(body);
        }
        let mut response = builder.send().map_err(|error| {
            if error.is_timeout() {
                ("timeout", "HTTP request timed out")
            } else {
                ("transport", "HTTP request failed")
            }
        })?;
        data["status"] = json!(response.status().as_u16());
        if request.include_headers {
            let mut headers = Map::new();
            let mut bytes = 0;
            for name in response.headers().keys() {
                let mut values = Vec::new();
                for value in response.headers().get_all(name) {
                    bytes += name.as_str().len() + value.as_bytes().len();
                    if bytes > MAX_RESPONSE_HEADERS {
                        return Err(("headers_limit", "HTTP response headers exceed 64 KiB"));
                    }
                    values.push(Value::String(
                        String::from_utf8_lossy(value.as_bytes()).into_owned(),
                    ));
                }
                headers.insert(name.as_str().into(), Value::Array(values));
            }
            data["headers"] = Value::Object(headers);
        }
        if response
            .content_length()
            .is_some_and(|length| length > MAX_BODY as u64)
        {
            data["truncated"] = json!(true);
            return Err(("body_limit", "HTTP response body exceeds 4 MiB"));
        }
        let mut bytes = Vec::new();
        let mut chunk = [0_u8; 16 * 1024];
        loop {
            if Instant::now() >= request.deadline {
                return Err(("timeout", "HTTP response body timed out"));
            }
            let read = response.read(&mut chunk).map_err(|error| {
                if error.kind() == std::io::ErrorKind::TimedOut
                    || Instant::now() >= request.deadline
                {
                    ("timeout", "HTTP response body timed out")
                } else {
                    ("body", "Cannot read HTTP response body")
                }
            })?;
            if read == 0 {
                break;
            }
            if bytes.len() + read > MAX_BODY {
                data["truncated"] = json!(true);
                return Err(("body_limit", "HTTP response body exceeds 4 MiB"));
            }
            bytes.extend_from_slice(&chunk[..read]);
        }
        match String::from_utf8(bytes) {
            Ok(body) => data["body"] = json!(body),
            Err(error) => data["body_base64"] = json!(base64(error.as_bytes())),
        }
        data["ok"] = json!(response.status().is_success());
        Ok(())
    })();
    if let Err((kind, error)) = result {
        data["error_kind"] = json!(kind);
        data["error"] = json!(error);
    }
    data["elapsed_ms"] = json!(request.submitted.elapsed().as_millis() as u64);
    HttpResponse {
        plugin_id: request.plugin_id,
        data,
    }
}

fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);
        output.push(ALPHABET[(first >> 2) as usize] as char);
        output.push(ALPHABET[(((first & 3) << 4) | (second >> 4)) as usize] as char);
        output.push(if chunk.len() > 1 {
            ALPHABET[(((second & 15) << 2) | (third >> 6)) as usize] as char
        } else {
            '='
        });
        output.push(if chunk.len() > 2 {
            ALPHABET[(third & 63) as usize] as char
        } else {
            '='
        });
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::Write,
        net::{TcpListener, TcpStream},
        sync::mpsc,
    };

    fn server(action: impl FnOnce(TcpStream) + Send + 'static) -> (String, thread::JoinHandle<()>) {
        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/resource", server.local_addr().unwrap());
        let task = thread::spawn(move || {
            let (stream, _) = server.accept().unwrap();
            action(stream);
        });
        (url, task)
    }
    fn drain_request(stream: &mut TcpStream) -> Vec<u8> {
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut bytes = Vec::new();
        let mut byte = [0];
        while !bytes.ends_with(b"\r\n\r\n") {
            stream.read_exact(&mut byte).unwrap();
            bytes.push(byte[0]);
        }
        bytes
    }
    fn wait(client: &HttpClient) -> HttpResponse {
        let started = Instant::now();
        loop {
            if let Some(response) = client.poll().pop() {
                return response;
            }
            assert!(started.elapsed() < Duration::from_secs(3));
            thread::sleep(Duration::from_millis(5));
        }
    }

    fn local_client() -> HttpClient {
        // Tests must never inherit a developer's HTTP_PROXY configuration.
        HttpClient::with_client(Ok(Client::builder().no_proxy().build().unwrap()))
    }

    #[test]
    fn sends_method_headers_body_and_preserves_non_success_response() {
        let (sent, received) = mpsc::channel();
        let (url, task) = server(move |mut stream| {
            let request = drain_request(&mut stream);
            let mut body = [0; 5];
            stream.read_exact(&mut body).unwrap();
            sent.send((String::from_utf8(request).unwrap(), body))
                .unwrap();
            stream.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 7\r\nX-Example: one\r\nX-Example: two\r\nConnection: close\r\n\r\nmissing").unwrap();
        });
        let client = local_client();
        client.submit("sample.plugin".into(),json!({"request_id":"sample.plugin:test","url":url,"method":"POST","headers":{"X-Sent":"example"},"body":"hello","include_headers":true})).unwrap();
        let response = wait(&client);
        assert_eq!(response.plugin_id, "sample.plugin");
        assert_eq!(response.data["status"], 404);
        assert_eq!(response.data["ok"], false);
        assert_eq!(response.data["body"], "missing");
        assert!(response.data["error"].is_null());
        assert_eq!(response.data["headers"]["x-example"], json!(["one", "two"]));
        let (request, body) = received.recv().unwrap();
        assert!(request.starts_with("POST /resource HTTP/1.1"));
        assert!(request.to_lowercase().contains("x-sent: example"));
        assert_eq!(&body, b"hello");
        task.join().unwrap();
    }

    #[test]
    fn body_timeout_applies_after_headers_arrive() {
        let (url, task) = server(|mut stream| {
            drain_request(&mut stream);
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\n")
                .unwrap();
            thread::sleep(Duration::from_millis(250));
            let _ = stream.write_all(b"later");
        });
        let client = local_client();
        let started = Instant::now();
        client
            .submit(
                "timeout".into(),
                json!({"request_id":"timeout:one","url":url,"timeout_ms":50}),
            )
            .unwrap();
        let response = wait(&client);
        assert_eq!(response.data["error_kind"], "timeout");
        assert!(started.elapsed() < Duration::from_millis(500));
        task.join().unwrap();
    }

    #[test]
    fn enforces_body_limit_without_content_length() {
        let (url, task) = server(|mut stream| {
            drain_request(&mut stream);
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n")
                .unwrap();
            let _ = stream.write_all(&vec![b'x'; MAX_BODY + 1]);
        });
        let client = local_client();
        client
            .submit("large".into(), json!({"request_id":"large:one","url":url}))
            .unwrap();
        let response = wait(&client);
        assert_eq!(response.data["error_kind"], "body_limit");
        assert_eq!(response.data["truncated"], true);
        assert!(response.data["body"].is_null());
        task.join().unwrap();
    }

    #[test]
    fn binary_body_is_lossless_base64() {
        let (url, task) = server(|mut stream| {
            drain_request(&mut stream);
            stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\nConnection: close\r\n\r\n\xff\x00\x80").unwrap();
        });
        let client = local_client();
        client
            .submit(
                "binary".into(),
                json!({"request_id":"binary:one","url":url}),
            )
            .unwrap();
        let response = wait(&client);
        assert_eq!(response.data["body_base64"], "/wCA");
        assert!(response.data["body"].is_null());
        task.join().unwrap();
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
    }

    #[test]
    fn rejects_invalid_requests_and_queued_deadlines_before_network_io() {
        for request in [
            json!({"request_id":"one","url":"file:///tmp/file"}),
            json!({"request_id":"one","url":"http://localhost","headers":{"X-Test":"bad\r\nvalue"}}),
            json!({"request_id":"one","url":"http://localhost","body":false}),
            json!({"request_id":"one","url":"http://localhost","timeout_ms":0}),
        ] {
            assert!(validate_request(&request).is_err());
        }
        let mut request = parse_request(
            "expired".into(),
            &json!({"request_id":"expired:one","url":"http://127.0.0.1:1"}),
        )
        .unwrap();
        request.deadline = Instant::now() - Duration::from_millis(1);
        let response = execute(&Client::new(), request);
        assert_eq!(response.data["error_kind"], "timeout");
    }
}
