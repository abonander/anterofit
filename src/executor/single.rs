use super::{ExecBox, Executor};

use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, SendError};
use std::thread;

type Sender = mpsc::Sender<Box<ExecBox>>;

/// An executor which completes all requests on a single background thread.
///
/// Use this for when you have a low volume of asynchronous requests.
///
/// If a panic occurs on the worker thread, it will automatically be restarted.
#[derive(Clone)]
pub struct SingleThread {
    master: Arc<Master>,
}

impl SingleThread {
    /// Construct a new executor, spawning a new background thread which will wait for tasks.
    pub fn new() -> Self {
        SingleThread {
            master: Arc::new(Master::new()),
        }
    }

    fn sender(&self) -> Sender {
        self.master.sender()
    }

    fn respawn(&self) {
        self.master.respawn();
    }
}

impl Executor for SingleThread {
    fn execute(&self, mut exec: Box<ExecBox>) {
        while let Err(SendError(exec_)) = self.sender().send(exec) {
            exec = exec_;
            self.respawn();
        }
    }
}

struct Master {
    sender: RwLock<SenderCell>,
    respawning: AtomicBool,
}

impl Master {
    fn new() -> Self {
        Master {
            sender: RwLock::new(SenderCell(spawn_thread())),
            respawning: AtomicBool::new(false),
        }
    }

    fn sender(&self) -> Sender {
        match self.sender.read() {
            Ok(sender) => sender.get(),
            Err(poisoned) => poisoned.into_inner().get(),
        }
    }

    fn respawn(&self) {
        // Lock with an atomic flag so we don't attempt to spawn more than one thread concurrently.
        if !self.respawning.compare_and_swap(false, true, Ordering::AcqRel) {
            let res = self.sender.write();

            let tx = spawn_thread();

            match res {
                Ok(mut sender) => sender.set(tx),
                Err(poisoned) => poisoned.into_inner().set(tx),
            }

            self.respawning.store(false, Ordering::Release);
        } else {
            // If we didn't get to do the respawn, just block until the respawn is done.
            let _ = self.sender.read();
        }
    }
}

fn spawn_thread() -> Sender {
    let (tx, rx) = mpsc::channel::<Box<ExecBox>>();

    thread::spawn(move ||
        for exec in rx {
            exec.exec();
        }
    );

    tx
}

struct SenderCell(Sender);

// `Sender` is not `Sync` for various other reasons. `Sender::clone()` is safe to call concurrently
// because it simply calls `Arc<_>::clone()`.
unsafe impl Sync for SenderCell {}

impl SenderCell {
    fn get(&self) -> Sender {
        self.0.clone()
    }

    fn set(&mut self, sender: Sender) {
        self.0 = sender;
    }
}