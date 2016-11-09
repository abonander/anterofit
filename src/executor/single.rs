use super::{ExecBox, Executor};

use std::sync::{Arc, RwLock};
use std::sync::mpsc::{self, SendError};
use std::thread;

type Sender = mpsc::Sender<Box<ExecBox>>;

#[derive(Clone)]
pub struct SingleThread {
    master: Arc<RwLock<MasterSender>>,
}

impl SingleThread {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<Box<ExecBox>>();

        thread::spawn(move || {
            for exec in rx {
                exec.exec();
            }
        });

        SingleThread {
            master: Arc::new(RwLock::new(MasterSender(tx))),
        }
    }

    fn sender(&self) -> Sender {
        match self.master.read() {
            Ok(master) => master.get(),
            Err(poisoned) => poisoned.into_inner().get(),
        }
    }

    fn respawn(&self) {
        let (tx, rx) = mpsc::channel::<Box<ExecBox>>();

        thread::spawn(move || {
            for exec in rx {
                exec.exec();
            }
        });

        self.set_sender(tx);
    }

    fn set_sender(&self, sender: Sender) {
        match self.master.write() {
            Ok(mut master) => master.set(sender),
            Err(poisoned) => poisoned.into_inner().set(sender),
        }
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

struct MasterSender(Sender);

// `Sender` is not `Sync` for various other reasons. `Sender::clone()` is safe to call concurrently.
unsafe impl Sync for MasterSender{}

impl MasterSender {
    fn get(&self) -> Sender {
        self.0.clone()
    }

    fn set(&mut self, sender: Sender) {
        self.0 = sender;
    }
}