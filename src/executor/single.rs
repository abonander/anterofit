use super::{ExecBox, Executor};

use std::sync::mpsc;
use std::thread::{self, Builder};

type Sender = mpsc::Sender<Box<ExecBox>>;
type Receiver = mpsc::Receiver<Box<ExecBox>>;

/// An executor which completes all requests in FIFO order on a single background thread.
///
/// Use this for when you have a low volume of asynchronous requests.
///
/// If a panic occurs on the worker thread, it will automatically be restarted; subsequent jobs will
/// be completed as normal.
#[derive(Clone)]
pub struct SingleThread {
    sender: Sender,
}

impl SingleThread {
    /// Construct a new executor, spawning a new background thread which will wait for tasks.
    ///
    /// The worker thread will be named such that it can be easily identified.
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<Box<ExecBox>>();

        spawn_thread(rx);

        SingleThread {
            sender: tx,
        }
    }
}

impl Executor for SingleThread {
    fn execute(&self, exec: Box<ExecBox>) {
        self.sender.send(exec)
            .expect("Worker thread unavailable for an unknown reason.");
    }
}

struct Sentinel(Option<Receiver>);

impl Drop for Sentinel {
    fn drop(&mut self) {
        if thread::panicking() {
            self.0.take().map(spawn_thread);
        }
    }
}

fn spawn_thread(rx: Receiver) {
    let sentinel = Sentinel(Some(rx));

    Builder::new()
        .name("anterofit_worker".into())
        .spawn(move ||
            for exec in sentinel.0.as_ref().unwrap() {
                exec.exec();
            }
        )
        .expect("Failed to spawn worker thread");
}