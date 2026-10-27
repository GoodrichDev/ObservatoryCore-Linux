//! Updates for compiled Linux plugins. A Windows .eop cannot update native Rust
//! code; this feed advertises compatible native worker artifacts individually.
use anyhow::{bail, ensure, Context, Result};
use observatory_io::{ConfigPaths, Settings};
use reqwest::{blocking::Client, Url};
use ring::digest::{self, Context as Hash};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, SyncSender},
    thread,
    time::Duration,
};
pub const PLUGINS: [&str; 4] = ["BioInsights", "Evaluator", "Hecate", "StellarOverlay"];
const MAX_MANIFEST: u64 = 2 * 1024 * 1024;
const MAX_ARTIFACT: u64 = 256 * 1024 * 1024;
pub fn platform() -> String {
    format!("{}-unknown-linux-gnu", std::env::consts::ARCH)
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Artifact {
    pub plugin: String,
    pub version: String,
    pub revision: u64,
    pub channel: String,
    pub url: String,
    pub sha256: String,
    pub size: u64,
    #[serde(default = "protocol")]
    pub protocol: u32,
}
fn protocol() -> u32 {
    1
}
#[derive(Debug, Deserialize)]
struct Manifest {
    schema: u32,
    platform: String,
    core_min: String,
    plugins: Vec<Artifact>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Installed {
    pub schema: u32,
    pub platform: String,
    pub core_min: String,
    pub artifact: Artifact,
    #[serde(default)]
    pub activated: bool,
}
fn safe_url(value: &str) -> Result<Url> {
    let url = Url::parse(value)?;
    let local = url.host_str().is_some_and(|s| {
        s == "localhost" || s.parse::<std::net::IpAddr>().is_ok_and(|a| a.is_loopback())
    });
    ensure!(
        (url.scheme() == "https" || url.scheme() == "http" && local)
            && url.username().is_empty()
            && url.password().is_none(),
        "Native updates require HTTPS (loopback HTTP is supported for local feeds)"
    );
    Ok(url)
}
fn version(value: &str) -> Result<Vec<u64>> {
    let parts = value
        .split('.')
        .map(str::parse::<u64>)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    ensure!((2..=4).contains(&parts.len()), "Invalid plugin version");
    Ok(parts)
}
fn source_version(name: &str) -> &'static str {
    match name {
        "BioInsights" => "1.3.444.1052",
        "Evaluator" => "1.3.444.637",
        "Hecate" => "1.3.444.761",
        _ => "1.3.228.387",
    }
}
fn compatible(manifest: &Manifest) -> Result<()> {
    ensure!(manifest.schema == 1, "Unsupported native update schema");
    ensure!(
        manifest.platform == platform(),
        "Update feed targets a different platform"
    );
    ensure!(
        semver::Version::parse(&manifest.core_min)?
            <= semver::Version::parse(env!("CARGO_PKG_VERSION"))?,
        "Native update requires a newer Core"
    );
    Ok(())
}
fn directory(paths: &ConfigPaths, plugin: &str) -> Result<PathBuf> {
    ensure!(PLUGINS.contains(&plugin), "Unknown native plugin");
    Ok(paths.data_dir.join("native-plugins").join(plugin))
}
fn digest_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn hash_file(path: &Path) -> Result<String> {
    let mut input = fs::File::open(path)?;
    let mut hash = Hash::new(&digest::SHA256);
    let mut bytes = [0; 65536];
    loop {
        let n = input.read(&mut bytes)?;
        if n == 0 {
            break;
        }
        hash.update(&bytes[..n]);
    }
    Ok(digest_hex(hash.finish().as_ref()))
}
fn validate_elf(path: &Path) -> Result<()> {
    let mut input = fs::File::open(path)?;
    let mut header = [0u8; 64];
    input.read_exact(&mut header)?;
    ensure!(
        &header[..4] == b"\x7fELF" && header[4] == 2 && header[5] == 1,
        "Downloaded plugin is not a native 64-bit Linux ELF"
    );
    let expected = match std::env::consts::ARCH {
        "x86_64" => 62,
        "aarch64" => 183,
        _ => bail!("Unsupported native update architecture"),
    };
    ensure!(
        u16::from_le_bytes([header[18], header[19]]) == expected,
        "Downloaded plugin has a different CPU architecture"
    );
    ensure!(
        [2, 3].contains(&u16::from_le_bytes([header[16], header[17]])),
        "Downloaded plugin is not an executable"
    );
    Ok(())
}
pub fn installed(paths: &ConfigPaths, plugin: &str) -> Result<Option<Installed>> {
    let root = directory(paths, plugin)?;
    let path = root.join("installed.json");
    if !path.is_file() {
        return Ok(None);
    }
    let metadata: Installed = serde_json::from_reader(fs::File::open(path)?)?;
    ensure!(
        metadata.schema == 1
            && metadata.platform == platform()
            && metadata.artifact.plugin == plugin
            && metadata.artifact.protocol == 1,
        "Installed native plugin metadata is incompatible"
    );
    ensure!(
        semver::Version::parse(&metadata.core_min)?
            <= semver::Version::parse(env!("CARGO_PKG_VERSION"))?,
        "Installed plugin requires newer Core"
    );
    let worker = root.join("worker");
    validate_elf(&worker)?;
    ensure!(
        fs::metadata(&worker)?.len() == metadata.artifact.size
            && hash_file(&worker)?.eq_ignore_ascii_case(&metadata.artifact.sha256),
        "Installed native plugin checksum does not match metadata"
    );
    Ok(Some(metadata))
}
pub fn worker_path(paths: &ConfigPaths, plugin: &str) -> Result<Option<PathBuf>> {
    Ok(installed(paths, plugin)?.map(|_| {
        paths
            .data_dir
            .join("native-plugins")
            .join(plugin)
            .join("worker")
    }))
}
pub fn mark_activated(paths: &ConfigPaths, plugin: &str) -> Result<bool> {
    let root = directory(paths, plugin)?;
    let path = root.join("installed.json");
    let mut metadata: Installed = serde_json::from_reader(fs::File::open(&path)?)?;
    if metadata.activated {
        return Ok(false);
    }
    metadata.activated = true;
    let mut file = tempfile::NamedTempFile::new_in(&root)?;
    serde_json::to_writer_pretty(&mut file, &metadata)?;
    file.as_file().sync_all()?;
    file.persist(path)?;
    Ok(true)
}
struct Request {
    url: String,
    plugin: String,
    beta: bool,
    install: bool,
}
pub struct Updates {
    tx: SyncSender<Request>,
    rx: Receiver<Value>,
    paths: ConfigPaths,
    feed: String,
    policies: BTreeMap<String, (bool, bool)>,
    states: BTreeMap<String, Value>,
}
impl Updates {
    pub fn new(paths: &ConfigPaths) -> Self {
        let (tx, input) = mpsc::sync_channel::<Request>(16);
        let (output, rx) = mpsc::channel();
        let directory = paths.clone();
        thread::Builder::new().name("observatory-native-updates".into()).spawn(move||{
            let client=Client::builder().user_agent(concat!("ObservatoryCore-Linux/",env!("CARGO_PKG_VERSION"))).connect_timeout(Duration::from_secs(10)).timeout(Duration::from_secs(120)).redirect(reqwest::redirect::Policy::custom(|attempt|{if attempt.previous().len()>=5||safe_url(attempt.url().as_str()).is_err(){attempt.error("Unsupported native update redirect")}else{attempt.follow()}})).build();
            while let Ok(request)=input.recv(){let result=client.as_ref().map_err(|_|anyhow::anyhow!("Cannot initialize native update transport")).and_then(|client|check(client,&directory,&request,&output));if let Err(error)=result{let _=output.send(json!({"plugin":request.plugin,"state":"Error","error":format!("{error:#}")}));}}
        }).expect("native update thread");
        Self {
            tx,
            rx,
            paths: paths.clone(),
            feed: String::new(),
            policies: BTreeMap::new(),
            states: BTreeMap::new(),
        }
    }
    pub fn configure(&mut self, settings: &Settings) {
        let feed = settings
            .extra
            .get("native_update_manifest_url")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        let changed = feed != self.feed;
        self.feed = feed;
        for plugin in PLUGINS {
            let install = settings.plugins[plugin]["InstallUpdates"]
                .as_bool()
                .unwrap_or(true);
            let beta = settings.plugins[plugin]["InstallBetas"]
                .as_bool()
                .unwrap_or(false);
            if changed || self.policies.get(plugin) != Some(&(install, beta)) {
                self.policies.insert(plugin.into(), (install, beta));
                if self.feed.is_empty() {
                    self.states.insert(
                        plugin.into(),
                        json!({"plugin":plugin,"state":"NoFeedConfigured"}),
                    );
                } else {
                    self.check(plugin, beta, install);
                }
            }
        }
    }
    pub fn check(&mut self, plugin: &str, beta: bool, install: bool) {
        if !PLUGINS.contains(&plugin) {
            return;
        }
        if self.feed.is_empty() {
            self.states.insert(
                plugin.into(),
                json!({"plugin":plugin,"state":"NoFeedConfigured"}),
            );
            return;
        }
        if self
            .tx
            .try_send(Request {
                url: self.feed.clone(),
                plugin: plugin.into(),
                beta,
                install,
            })
            .is_ok()
        {
            self.states
                .insert(plugin.into(), json!({"plugin":plugin,"state":"Checking"}));
        } else {
            self.states.insert(
                plugin.into(),
                json!({"plugin":plugin,"state":"Error","error":"Native update queue is full"}),
            );
        }
    }
    pub fn install(&mut self, plugin: &str) {
        let beta = self.policies.get(plugin).map(|p| p.1).unwrap_or(false);
        self.check(plugin, beta, true);
    }
    pub fn poll(&mut self) -> Vec<Value> {
        let events = self.rx.try_iter().take(64).collect::<Vec<_>>();
        for event in &events {
            if let Some(plugin) = event["plugin"].as_str() {
                self.states.insert(plugin.into(), event.clone());
            }
        }
        events
    }
    pub fn summary(&self) -> Value {
        json!({"configured":!self.feed.is_empty(),"plugins":self.states,"directory":self.paths.data_dir.join("native-plugins")})
    }
}
fn check(
    client: &Client,
    paths: &ConfigPaths,
    request: &Request,
    output: &mpsc::Sender<Value>,
) -> Result<()> {
    let response = client
        .get(safe_url(&request.url)?)
        .send()
        .map_err(|e| e.without_url())?
        .error_for_status()
        .map_err(|e| e.without_url())?;
    let mut bytes = Vec::new();
    response.take(MAX_MANIFEST + 1).read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 <= MAX_MANIFEST,
        "Native update manifest is too large"
    );
    let manifest: Manifest = serde_json::from_slice(&bytes)?;
    compatible(&manifest)?;
    let selected = manifest
        .plugins
        .iter()
        .filter(|a| {
            a.plugin == request.plugin
                && (a.channel == "stable" || request.beta && a.channel == "beta")
        })
        .map(|a| Ok((version(&a.version)?, a.revision, a)))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .max_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)))
        .map(|(_, _, a)| a);
    let current = installed(paths, &request.plugin)?;
    let local_version = version(
        current
            .as_ref()
            .map(|i| i.artifact.version.as_str())
            .unwrap_or(source_version(&request.plugin)),
    )?;
    let local_revision = current.as_ref().map(|i| i.artifact.revision).unwrap_or(1);
    let Some(artifact) = selected.filter(|a| {
        version(&a.version)
            .is_ok_and(|v| v > local_version || v == local_version && a.revision > local_revision)
    }) else {
        if let Some(current) = current.as_ref().filter(|m| !m.activated) {
            let _=output.send(json!({"plugin":request.plugin,"state":"Downloaded","version":current.artifact.version,"revision":current.artifact.revision,"restart_required":true}));
        } else {
            let _ = output.send(json!({"plugin":request.plugin,"state":"UpToDate"}));
        }
        return Ok(());
    };
    ensure!(
        artifact.protocol == 1
            && artifact.size >= 64
            && artifact.size <= MAX_ARTIFACT
            && artifact.sha256.len() == 64
            && artifact.sha256.bytes().all(|b| b.is_ascii_hexdigit()),
        "Invalid native worker artifact metadata"
    );
    let _=output.send(json!({"plugin":request.plugin,"state":"Available","version":artifact.version,"revision":artifact.revision}));
    if !request.install {
        return Ok(());
    }
    let root = directory(paths, &request.plugin)?;
    fs::create_dir_all(&root)?;
    let mut file = tempfile::NamedTempFile::new_in(&root)?;
    let mut response = client
        .get(safe_url(&artifact.url)?)
        .send()
        .map_err(|e| e.without_url())?
        .error_for_status()
        .map_err(|e| e.without_url())?;
    let mut hash = Hash::new(&digest::SHA256);
    let mut buffer = [0; 65536];
    let mut size = 0;
    loop {
        let n = response.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        size += n as u64;
        ensure!(
            size <= artifact.size && size <= MAX_ARTIFACT,
            "Native worker download exceeds its declared size"
        );
        hash.update(&buffer[..n]);
        file.write_all(&buffer[..n])?;
    }
    ensure!(
        size == artifact.size
            && digest_hex(hash.finish().as_ref()).eq_ignore_ascii_case(&artifact.sha256),
        "Native worker download checksum or size does not match manifest"
    );
    validate_elf(file.path())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.as_file()
            .set_permissions(fs::Permissions::from_mode(0o700))?;
    }
    file.as_file().sync_all()?;
    let file = file.into_temp_path();
    let hello = observatory_plugins::worker_rpc::inspect(&file, request.plugin.parse()?)?;
    ensure!(
        hello.source_version == artifact.version,
        "Native worker version does not match manifest"
    );
    let metadata = Installed {
        schema: 1,
        platform: manifest.platform,
        core_min: manifest.core_min,
        artifact: artifact.clone(),
        activated: false,
    };
    let mut metadata_file = tempfile::NamedTempFile::new_in(&root)?;
    serde_json::to_writer_pretty(&mut metadata_file, &metadata)?;
    metadata_file.as_file().sync_all()?;
    let worker = root.join("worker");
    let manifest_path = root.join("installed.json");
    let backup = worker.with_extension("previous");
    let previous = manifest_path.with_extension("previous.json");
    let had_worker = worker.is_file();
    let had_manifest = manifest_path.is_file();
    if had_worker {
        fs::copy(&worker, &backup)?;
    }
    if had_manifest {
        fs::copy(&manifest_path, &previous)?;
    }
    file.persist(&worker)?;
    if let Err(error) = metadata_file.persist(&manifest_path) {
        if had_worker {
            fs::rename(&backup, &worker)?;
        } else {
            fs::remove_file(&worker)?;
        }
        return Err(error).context("Cannot commit native worker metadata");
    }
    let _=output.send(json!({"plugin":request.plugin,"state":"Downloaded","version":artifact.version,"revision":artifact.revision,"restart_required":true}));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        net::TcpListener,
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc, Mutex,
        },
    };
    struct Server {
        url: String,
        files: Arc<Mutex<BTreeMap<String, Vec<u8>>>>,
        requests: Arc<Mutex<Vec<String>>>,
        stop: Arc<AtomicBool>,
        thread: Option<thread::JoinHandle<()>>,
    }
    impl Server {
        fn new() -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            listener.set_nonblocking(true).unwrap();
            let url = format!("http://{}", listener.local_addr().unwrap());
            let files = Arc::new(Mutex::new(BTreeMap::<String, Vec<u8>>::new()));
            let requests = Arc::new(Mutex::new(Vec::new()));
            let stop = Arc::new(AtomicBool::new(false));
            let (f, r, s) = (files.clone(), requests.clone(), stop.clone());
            let thread = thread::spawn(move || {
                while !s.load(Ordering::Relaxed) {
                    match listener.accept() {
                        Ok((mut stream, _)) => {
                            stream
                                .set_read_timeout(Some(Duration::from_secs(2)))
                                .unwrap();
                            let mut request = Vec::new();
                            let mut byte = [0];
                            while request.len() < 16384 && stream.read(&mut byte).unwrap_or(0) == 1
                            {
                                request.push(byte[0]);
                                if request.ends_with(b"\r\n\r\n") {
                                    break;
                                }
                            }
                            let request = String::from_utf8_lossy(&request);
                            let path = request.split_whitespace().nth(1).unwrap_or("").to_owned();
                            r.lock().unwrap().push(path.clone());
                            let body = f.lock().unwrap().get(&path).cloned().unwrap_or_default();
                            let _=write!(stream,"HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",body.len());
                            let _ = stream.write_all(&body);
                        }
                        Err(_) => thread::sleep(Duration::from_millis(2)),
                    }
                }
            });
            Self {
                url,
                files,
                requests,
                stop,
                thread: Some(thread),
            }
        }
        fn manifest(&self, artifact: &Artifact) {
            self.files.lock().unwrap().insert("/manifest".into(),serde_json::to_vec(&json!({"schema":1,"platform":platform(),"core_min":"2.0.0","plugins":[artifact]})).unwrap());
        }
        fn artifact(&self, bytes: Vec<u8>, channel: &str) -> Artifact {
            let artifact = Artifact {
                plugin: "Evaluator".into(),
                version: source_version("Evaluator").into(),
                revision: 2,
                channel: channel.into(),
                url: format!("{}/worker", self.url),
                sha256: digest_hex(digest::digest(&digest::SHA256, &bytes).as_ref()),
                size: bytes.len() as u64,
                protocol: 1,
            };
            self.files.lock().unwrap().insert("/worker".into(), bytes);
            artifact
        }
    }
    impl Drop for Server {
        fn drop(&mut self) {
            self.stop.store(true, Ordering::Relaxed);
            if let Some(thread) = self.thread.take() {
                thread.join().unwrap();
            }
        }
    }
    fn paths(temp: &Path) -> ConfigPaths {
        let paths =
            ConfigPaths::from_roots(temp.join("config"), temp.join("data"), temp.join("cache"));
        paths.create_dirs().unwrap();
        paths
    }
    #[test]
    #[ignore = "requires OBSERVATORY_NATIVE_WORKER_TEST_BINARY; run the real worker updater integration"]
    fn native_download_checks_hash_elf_metadata_and_stages_only_selected_plugin() {
        let temp = tempfile::tempdir().unwrap();
        let paths = paths(temp.path());
        let server = Server::new();
        let bytes = fs::read(
            std::env::var_os("OBSERVATORY_NATIVE_WORKER_TEST_BINARY")
                .expect("real native worker binary"),
        )
        .unwrap();
        let artifact = server.artifact(bytes.clone(), "stable");
        server.manifest(&artifact);
        let (tx, rx) = mpsc::channel();
        check(
            &Client::new(),
            &paths,
            &Request {
                url: format!("{}/manifest", server.url),
                plugin: "Evaluator".into(),
                beta: false,
                install: true,
            },
            &tx,
        )
        .unwrap();
        let states = rx
            .try_iter()
            .map(|v| v["state"].as_str().unwrap().to_owned())
            .collect::<Vec<_>>();
        assert_eq!(states, ["Available", "Downloaded"]);
        assert_eq!(
            fs::read(worker_path(&paths, "Evaluator").unwrap().unwrap()).unwrap(),
            bytes
        );
        assert_eq!(
            installed(&paths, "Evaluator")
                .unwrap()
                .unwrap()
                .artifact
                .revision,
            2
        );
        for plugin in ["BioInsights", "Hecate", "StellarOverlay"] {
            assert!(!directory(&paths, plugin).unwrap().exists());
        }
        // A corrupt subsequent artifact cannot replace the previously verified
        // worker or metadata. It does not emit a successful download event.
        let mut broken = artifact;
        broken.revision = 3;
        broken.sha256 = "0".repeat(64);
        server.manifest(&broken);
        assert!(check(
            &Client::new(),
            &paths,
            &Request {
                url: format!("{}/manifest", server.url),
                plugin: "Evaluator".into(),
                beta: false,
                install: true
            },
            &tx
        )
        .is_err());
        assert_eq!(
            installed(&paths, "Evaluator")
                .unwrap()
                .unwrap()
                .artifact
                .revision,
            2
        );
        assert_eq!(
            fs::read(directory(&paths, "Evaluator").unwrap().join("worker")).unwrap(),
            bytes
        );
        assert!(!rx.try_iter().any(|v| v["state"] == "Downloaded"));
    }
    #[test]
    fn beta_and_manual_install_policies_do_not_download_unselected_artifacts() {
        let temp = tempfile::tempdir().unwrap();
        let paths = paths(temp.path());
        let server = Server::new();
        let artifact = server.artifact(fs::read("/bin/true").unwrap(), "beta");
        server.manifest(&artifact);
        let (tx, rx) = mpsc::channel();
        let mut request = Request {
            url: format!("{}/manifest", server.url),
            plugin: "Evaluator".into(),
            beta: false,
            install: true,
        };
        check(&Client::new(), &paths, &request, &tx).unwrap();
        assert_eq!(rx.try_recv().unwrap()["state"], "UpToDate");
        request.beta = true;
        request.install = false;
        check(&Client::new(), &paths, &request, &tx).unwrap();
        assert_eq!(rx.try_recv().unwrap()["state"], "Available");
        assert!(!server
            .requests
            .lock()
            .unwrap()
            .iter()
            .any(|p| p == "/worker"));
        assert!(!directory(&paths, "Evaluator").unwrap().exists());
        server.files.lock().unwrap().insert("/manifest".into(),serde_json::to_vec(&json!({"schema":1,"platform":"wrong-platform","core_min":"2.0.0","plugins":[artifact]})).unwrap());
        assert!(check(&Client::new(), &paths, &request, &tx).is_err());
    }
    #[test]
    fn incompatible_worker_and_path_traversal_are_rejected() {
        let temp = tempfile::tempdir().unwrap();
        let paths = paths(temp.path());
        assert!(directory(&paths, "../../escape").is_err());
        assert!(safe_url("http://example.com/worker").is_err());
        assert!(safe_url("https://user:password@example.com/worker").is_err());
        let server = Server::new();
        let artifact = server.artifact(
            b"MZ Windows DLL is not a native plugin executable".repeat(4),
            "stable",
        );
        server.manifest(&artifact);
        let (tx, _rx) = mpsc::channel();
        assert!(check(
            &Client::new(),
            &paths,
            &Request {
                url: format!("{}/manifest", server.url),
                plugin: "Evaluator".into(),
                beta: false,
                install: true
            },
            &tx
        )
        .is_err());
        assert!(!directory(&paths, "Evaluator")
            .unwrap()
            .join("worker")
            .exists());
    }
}
