//! Optional Hecate HTTP/WebSocket bridge. Network work never runs in journal callbacks.
use super::{sanitize_name, VERSION};
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, HashMap},
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{self, Receiver, SyncSender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tungstenite::{protocol::WebSocketConfig, Message};

const POLL: Duration = Duration::from_millis(20);
const TIMEOUT: Duration = Duration::from_secs(10);
const LIMIT: usize = 4 * 1024 * 1024;
type Registry = Arc<Mutex<BTreeMap<String, Value>>>;
type Clients = Arc<Mutex<HashMap<String, (String, SyncSender<String>)>>>;
type Responses = Arc<Mutex<HashMap<String, SyncSender<Value>>>>;

pub struct Server {
    address: String,
    port: u16,
    stop: Arc<AtomicBool>,
    plugins: Registry,
    clients: Clients,
    responses: Responses,
    events: Receiver<Value>,
    worker: Option<JoinHandle<()>>,
}

impl Server {
    pub fn start(address: &str, port: u16) -> Result<Self> {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, port))
            .context("bind Hecate localhost server")?;
        let port = listener.local_addr()?.port();
        listener.set_nonblocking(true)?;
        let mut listeners = vec![listener];
        let address = if address.is_empty() {
            "localhost".to_owned()
        } else {
            let ip: Ipv4Addr = address
                .parse()
                .context("Hecate LAN address must be an IPv4 address")?;
            if ip != Ipv4Addr::LOCALHOST {
                let listener = TcpListener::bind((ip, port)).context("bind Hecate LAN server")?;
                listener.set_nonblocking(true)?;
                listeners.push(listener);
            }
            address.to_owned()
        };
        let stop = Arc::new(AtomicBool::new(false));
        let plugins: Registry = Arc::default();
        let clients: Clients = Arc::default();
        let responses: Responses = Arc::default();
        let (send, events) = mpsc::sync_channel(128);
        let state = State {
            address: address.clone(),
            port,
            stop: stop.clone(),
            plugins: plugins.clone(),
            clients: clients.clone(),
            responses: responses.clone(),
            events: send,
            sequence: Arc::new(AtomicU64::new(1)),
        };
        let worker = thread::Builder::new()
            .name("hecate-listener".into())
            .spawn(move || {
                let mut workers: Vec<JoinHandle<()>> = Vec::new();
                while !state.stop.load(Ordering::Relaxed) {
                    let mut index = 0;
                    while index < workers.len() {
                        if workers[index].is_finished() {
                            let _ = workers.swap_remove(index).join();
                        } else {
                            index += 1;
                        }
                    }
                    for listener in &listeners {
                        while let Ok((mut socket, peer)) = listener.accept() {
                            if workers.len() >= 64 {
                                let _ = write_response(
                                    &mut socket,
                                    503,
                                    "text/plain",
                                    b"Server Busy",
                                    None,
                                );
                                continue;
                            }
                            let state = state.clone();
                            if let Ok(worker) = thread::Builder::new()
                                .name("hecate-client".into())
                                .spawn(move || {
                                    let _ = serve(socket, peer, state);
                                })
                            {
                                workers.push(worker);
                            }
                        }
                    }
                    thread::sleep(POLL);
                }
                for worker in workers {
                    let _ = worker.join();
                }
            })?;
        Ok(Self {
            address,
            port,
            stop,
            plugins,
            clients,
            responses,
            events,
            worker: Some(worker),
        })
    }
    pub fn port(&self) -> u16 {
        self.port
    }
    pub fn address(&self) -> &str {
        &self.address
    }
    pub fn register(&self, name: &str, target: Value) -> Result<()> {
        let name = sanitize_name(name);
        if name.is_empty() || matches!(name.as_str(), "status" | "debug" | "faviconico") {
            bail!("reserved or empty Hecate plugin name");
        }
        self.plugins.lock().unwrap().insert(name, target);
        Ok(())
    }
    pub fn poll(&self) -> Vec<Value> {
        self.events.try_iter().collect()
    }
    pub fn respond(&self, request: &str, response: Value) -> Result<()> {
        let sender = self
            .responses
            .lock()
            .unwrap()
            .remove(request)
            .context("Hecate web request expired or unknown")?;
        sender
            .try_send(response)
            .context("Hecate web response could not be delivered")
    }
    pub fn send(&self, client: &str, message: &str) -> Result<()> {
        if message.len() > LIMIT {
            bail!("Hecate WebSocket response too large");
        }
        let clients = self.clients.lock().unwrap();
        let (_, sender) = clients
            .get(client)
            .context("Hecate WebSocket client no longer connected")?;
        sender
            .try_send(message.to_owned())
            .context("Hecate WebSocket output queue full")
    }
    pub fn broadcast(&self, plugin: &str, message: &str) {
        if message.len() > LIMIT {
            return;
        }
        let plugin = sanitize_name(plugin);
        for (name, sender) in self.clients.lock().unwrap().values() {
            if *name == plugin {
                let _ = sender.try_send(message.to_owned());
            }
        }
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

#[derive(Clone)]
struct State {
    address: String,
    port: u16,
    stop: Arc<AtomicBool>,
    plugins: Registry,
    clients: Clients,
    responses: Responses,
    events: SyncSender<Value>,
    sequence: Arc<AtomicU64>,
}

fn serve(mut socket: TcpStream, _peer: SocketAddr, state: State) -> Result<()> {
    socket.set_read_timeout(Some(Duration::from_millis(200)))?;
    socket.set_write_timeout(Some(Duration::from_secs(2)))?;
    let mut buffer = vec![0; 32768];
    let started = Instant::now();
    let count = loop {
        if state.stop.load(Ordering::Relaxed) {
            return Ok(());
        }
        match socket.peek(&mut buffer) {
            Ok(0) => return Ok(()),
            Ok(count)
                if buffer[..count]
                    .windows(4)
                    .any(|window| window == b"\r\n\r\n") =>
            {
                break count
            }
            Ok(count) if count == buffer.len() => {
                return write_response(
                    &mut socket,
                    431,
                    "text/plain",
                    b"Request Headers Too Large",
                    None,
                )
            }
            Ok(_) => {}
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) => {}
            Err(error) => return Err(error.into()),
        }
        if started.elapsed() > TIMEOUT {
            return write_response(&mut socket, 408, "text/plain", b"Request Timeout", None);
        }
        thread::sleep(POLL);
    };
    let header =
        std::str::from_utf8(&buffer[..count]).context("Hecate HTTP headers are not UTF-8")?;
    let mut lines = header.split("\r\n");
    let mut first = lines.next().unwrap_or("").split_whitespace();
    let method = first.next().unwrap_or("");
    let uri = first.next().unwrap_or("");
    let path = uri.split('?').next().unwrap_or(uri);
    let headers: HashMap<String, String> = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(name, value)| (name.to_ascii_lowercase(), value.trim().to_owned()))
        .collect();
    // WebSocket accept must read its handshake itself; consume ordinary HTTP
    // headers so closing the stream does not reset a completed response.
    if method != "GET" || !path.starts_with("/ws/") {
        let header_len = buffer[..count]
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .unwrap()
            + 4;
        let mut consumed = vec![0; header_len];
        socket.read_exact(&mut consumed)?;
    }
    if method != "GET" {
        return write_response(&mut socket, 405, "text/plain", b"Method Not Allowed", None);
    }
    if let Some(plugin) = path.strip_prefix("/ws/") {
        if plugin.contains('/') {
            return write_response(&mut socket, 404, "text/plain", b"Plugin not found", None);
        }
        let plugin = sanitize_name(plugin);
        let target = state.plugins.lock().unwrap().get(&plugin).cloned();
        let Some(target) = target else {
            return write_response(&mut socket, 404, "text/plain", b"Plugin not found", None);
        };
        let config = WebSocketConfig::default()
            .max_message_size(Some(LIMIT))
            .max_frame_size(Some(LIMIT));
        let mut websocket = match tungstenite::accept_with_config(socket, Some(config)) {
            Ok(socket) => socket,
            Err(_) => return Ok(()),
        };
        websocket.get_mut().set_nonblocking(true)?;
        let client = client_id(state.sequence.fetch_add(1, Ordering::Relaxed));
        let (send, receive) = mpsc::sync_channel(128);
        state
            .clients
            .lock()
            .unwrap()
            .insert(client.clone(), (plugin, send));
        while !state.stop.load(Ordering::Relaxed) {
            for message in receive.try_iter() {
                if websocket.send(Message::Text(message.into())).is_err() {
                    break;
                }
            }
            match websocket.read() {
                Ok(Message::Text(message)) => {
                    if state.events.try_send(json!({"Target":target,"Command":"WebSocketRequest","Payload":{"URI":message.as_str(),"ClientID":client}})).is_err() { break; }
                },
                Ok(Message::Close(_)) => break,
                Ok(_) => {},
                Err(tungstenite::Error::Io(error)) if error.kind() == std::io::ErrorKind::WouldBlock => {},
                Err(_) => break,
            }
            match websocket.flush() {
                Err(tungstenite::Error::Io(error))
                    if error.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(_) => break,
                Ok(()) => {}
            }
            thread::sleep(POLL);
        }
        let _ = websocket.close(None);
        state.clients.lock().unwrap().remove(&client);
        return Ok(());
    }
    if path == "/status" {
        let plugins = state.plugins.lock().unwrap().clone();
        let client_lookup: BTreeMap<String, String> = state
            .clients
            .lock()
            .unwrap()
            .iter()
            .map(|(id, (plugin, _))| (id.clone(), plugin.clone()))
            .collect();
        let clients: BTreeMap<String,Value>=client_lookup.keys().map(|id|(id.clone(),json!({"CloseStatus":null,"CloseStatusDescription":null,"State":1,"SubProtocol":null}))).collect();
        let body = serde_json::to_vec(
            &json!({"status":"ok","plugins":plugins,"_clients":clients,"_clientLookup":client_lookup}),
        )?;
        return write_response(&mut socket, 200, "application/json", &body, None);
    }
    if path == "/favicon.ico" {
        return write_response(&mut socket, 200, "text/plain", b"", None);
    }
    if path == "/" {
        let mut body = format!("<html><header><title>Hecate Server v{VERSION}</title><style>a.button {{\r\n    padding: 6px 16px;\r\n    border: 2px outset buttonborder;\r\n    border-radius: 3px;\r\n    color: buttontext;\r\n    background-color: buttonface;\r\n    text-decoration: none;\r\n}}</style></header><body><div style=\"text-align: center;display:inline-block;\">Hecate Server v{VERSION}<br /><br />");
        for plugin in state.plugins.lock().unwrap().keys() {
            body.push_str(&format!(
                "<a class=\"button\" href=\"http://{}:{}/{plugin}\">{plugin}</a><br /><br />",
                state.address, state.port
            ));
        }
        body.push_str("</div></body></html>");
        return write_response(
            &mut socket,
            200,
            "text/html; charset=utf-8",
            body.as_bytes(),
            None,
        );
    }
    let plugin = sanitize_name(path.trim_start_matches('/').split('/').next().unwrap_or(""));
    let target = state.plugins.lock().unwrap().get(&plugin).cloned();
    let Some(target) = target else {
        return write_response(
            &mut socket,
            404,
            "text/plain",
            format!("Plugin not found [{plugin}]").as_bytes(),
            None,
        );
    };
    let host = headers
        .get("host")
        .cloned()
        .unwrap_or_else(|| "localhost".into());
    let request = format!("web:{}", state.sequence.fetch_add(1, Ordering::Relaxed));
    let (send, receive) = mpsc::sync_channel(1);
    state
        .responses
        .lock()
        .unwrap()
        .insert(request.clone(), send);
    if state.events.try_send(json!({"Target":target,"Command":"WebRequest","Payload":{"URI":format!("http://{host}{uri}"),"RequestID":request,"Response":{}}})).is_err() {
        state.responses.lock().unwrap().remove(&request);
        return write_response(&mut socket, 503, "text/plain", b"Server Busy", None);
    }
    let started = Instant::now();
    loop {
        if state.stop.load(Ordering::Relaxed) {
            break;
        }
        if let Ok(response) = receive.recv_timeout(POLL) {
            let status = response
                .get("StatusCode")
                .or_else(|| response.get("Status"))
                .unwrap_or(&Value::Null)
                .as_u64()
                .filter(|status| (100..=599).contains(status))
                .unwrap_or(if response.get("Location").is_some() {
                    302
                } else {
                    200
                }) as u16;
            let content_type = response["ContentType"]
                .as_str()
                .unwrap_or("text/html; charset=utf-8");
            let body = if let Some(text) = response["Content"].as_str() {
                text.as_bytes().to_vec()
            } else if let Some(bytes) = response["Content"].as_array() {
                bytes
                    .iter()
                    .map(|byte| {
                        byte.as_u64()
                            .filter(|b| *b <= 255)
                            .map(|b| b as u8)
                            .context("invalid image byte")
                    })
                    .collect::<Result<Vec<_>>>()?
            } else if response.get("Location").is_some() {
                Vec::new()
            } else {
                return write_response(&mut socket, 500, "text/plain", b"Empty Response", None);
            };
            if body.len() > LIMIT {
                return write_response(&mut socket, 500, "text/plain", b"Response Too Large", None);
            }
            return write_response(
                &mut socket,
                status,
                content_type,
                &body,
                response["Location"].as_str(),
            );
        }
        if started.elapsed() > TIMEOUT {
            break;
        }
    }
    state.responses.lock().unwrap().remove(&request);
    write_response(&mut socket, 504, "text/plain", b"Request Timeout", None)
}

fn write_response(
    socket: &mut TcpStream,
    status: u16,
    content_type: &str,
    content: &[u8],
    location: Option<&str>,
) -> Result<()> {
    let clean = |text: &str| !text.contains(['\r', '\n']);
    let content_type = if clean(content_type) {
        content_type
    } else {
        "application/octet-stream"
    };
    let mut header = format!("HTTP/1.1 {status} Response\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n",content.len());
    if let Some(location) = location.filter(|text| clean(text)) {
        header.push_str(&format!("Location: {location}\r\n"));
    }
    header.push_str("\r\n");
    socket.write_all(header.as_bytes())?;
    socket.write_all(content)?;
    Ok(())
}
fn client_id(sequence: u64) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let value = format!("{:032x}", timestamp ^ u128::from(sequence));
    format!(
        "{}-{}-{}-{}-{}",
        &value[..8],
        &value[8..12],
        &value[12..16],
        &value[16..20],
        &value[20..]
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    fn request(port: u16, path: &str) -> String {
        let mut socket = TcpStream::connect((Ipv4Addr::LOCALHOST, port)).unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        write!(
            socket,
            "GET {path} HTTP/1.1\r\nHost: localhost:{port}\r\n\r\n"
        )
        .unwrap();
        let mut body = String::new();
        socket.read_to_string(&mut body).unwrap();
        body
    }
    fn next(server: &Server) -> Value {
        let started = Instant::now();
        loop {
            if let Some(event) = server.poll().into_iter().next() {
                return event;
            }
            assert!(
                started.elapsed() < Duration::from_secs(3),
                "bridge event not delivered"
            );
            thread::sleep(POLL);
        }
    }
    #[test]
    fn serves_original_routes_and_dispatches_web_responses() {
        let server = Server::start("", 0).unwrap();
        server.register("BioInsights", json!("bio-guid")).unwrap();
        assert!(server.register("status", json!("reserved")).is_err());
        let index = request(server.port(), "/");
        assert!(index.contains(&format!(
            "href=\"http://localhost:{}/bioinsights\"",
            server.port()
        )));
        assert!(index.contains("a.button {\r\n    padding: 6px 16px;"));
        assert!(index.contains(&format!("<title>Hecate Server v{VERSION}</title>")));
        let status = request(server.port(), "/status");
        assert!(status.contains("\"status\":\"ok\""));
        assert!(status.contains("bio-guid"));
        assert!(request(server.port(), "/missing/path").starts_with("HTTP/1.1 404"));
        let port = server.port();
        let client = thread::spawn(move || request(port, "/bioinsights/data?mode=all"));
        let event = next(&server);
        assert_eq!(event["Target"], "bio-guid");
        assert_eq!(event["Command"], "WebRequest");
        assert!(event["Payload"]["URI"]
            .as_str()
            .unwrap()
            .ends_with("/bioinsights/data?mode=all"));
        server.respond(event["Payload"]["RequestID"].as_str().unwrap(),json!({"ContentType":"application/json","Content":"{\"actual\":true}","StatusCode":201})).unwrap();
        let response = client.join().unwrap();
        assert!(response.starts_with("HTTP/1.1 201"));
        assert!(response.ends_with("{\"actual\":true}"));
    }
    #[test]
    fn websocket_dispatch_response_broadcast_and_shutdown() {
        let server = Server::start("", 0).unwrap();
        server
            .register("Evaluator", json!("evaluator-guid"))
            .unwrap();
        let (mut socket, _) =
            tungstenite::connect(format!("ws://127.0.0.1:{}/ws/evaluator", server.port()).as_str())
                .unwrap();
        socket
            .send(Message::Text("http://localhost/evaluator/data".into()))
            .unwrap();
        let event = next(&server);
        assert_eq!(event["Target"], "evaluator-guid");
        assert_eq!(event["Command"], "WebSocketRequest");
        assert_eq!(event["Payload"]["URI"], "http://localhost/evaluator/data");
        let client = event["Payload"]["ClientID"].as_str().unwrap();
        assert_eq!(client.len(), 36);
        let status = request(server.port(), "/status");
        assert!(status.contains(client));
        assert!(status.contains("\"CloseStatus\":null"));
        assert!(status.contains("\"State\":1"));
        server.send(client, "{\"reply\":1}").unwrap();
        assert_eq!(socket.read().unwrap().into_text().unwrap(), "{\"reply\":1}");
        server.broadcast("Evaluator", "{\"broadcast\":2}");
        assert_eq!(
            socket.read().unwrap().into_text().unwrap(),
            "{\"broadcast\":2}"
        );
        let started = Instant::now();
        drop(server);
        assert!(started.elapsed() < Duration::from_secs(1));
        assert!(matches!(socket.read(), Ok(Message::Close(_)) | Err(_)));
    }
}
