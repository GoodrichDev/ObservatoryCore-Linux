//! Keep the original bounded route search away from journal/UI dispatch.
use super::{model::System, routing};
use serde_json::Value;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};
type Key = (String, u64);
struct Request {
    revision: u64,
    key: Key,
    system: System,
    settings: Value,
    time: String,
    cancel: Arc<AtomicBool>,
}
struct Result {
    revision: u64,
    key: Key,
    route: Vec<u64>,
}
pub(super) struct Worker {
    sender: mpsc::Sender<Request>,
    receiver: mpsc::Receiver<Result>,
    revision: u64,
    cancel: Arc<AtomicBool>,
    pending: bool,
}
impl Worker {
    pub fn new() -> Self {
        let (sender, requests) = mpsc::channel::<Request>();
        let (results, receiver) = mpsc::channel();
        std::thread::Builder::new()
            .name("evaluator-route".into())
            .spawn(move || {
                while let Ok(mut query) = requests.recv() {
                    while let Ok(next) = requests.try_recv() {
                        query = next;
                    }
                    if query.cancel.load(Ordering::Relaxed) {
                        continue;
                    }
                    routing::update_with_cancel(
                        &mut query.system,
                        &query.settings,
                        &query.time,
                        &query.cancel,
                    );
                    if !query.cancel.load(Ordering::Relaxed)
                        && results
                            .send(Result {
                                revision: query.revision,
                                key: query.key,
                                route: query.system.route,
                            })
                            .is_err()
                    {
                        break;
                    }
                }
            })
            .expect("Evaluator route worker thread");
        Self {
            sender,
            receiver,
            revision: 0,
            cancel: Arc::new(AtomicBool::new(false)),
            pending: false,
        }
    }
    pub fn cancel(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
        self.revision += 1;
        self.pending = false;
    }
    pub fn submit(&mut self, key: Key, system: System, settings: Value, time: String) {
        self.cancel();
        self.cancel = Arc::new(AtomicBool::new(false));
        self.pending = true;
        let _ = self.sender.send(Request {
            revision: self.revision,
            key,
            system,
            settings,
            time,
            cancel: self.cancel.clone(),
        });
    }
    pub fn pending(&self) -> bool {
        self.pending
    }
    pub fn poll(&mut self) -> Option<(Key, Vec<u64>)> {
        let mut answer = None;
        while let Ok(result) = self.receiver.try_recv() {
            if result.revision == self.revision {
                self.pending = false;
                answer = Some((result.key, result.route));
            }
        }
        answer
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}
