use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;

use ::ExecBox;

#[cfg(feature = "pool")]
mod pool;

#[cfg(feature = "pool")]
pub use pool::Pooled;

pub type DefaultExecutor = SingleThread;


pub trait Executor {
    fn execute(&self, exec: Box<ExecBox>);
}

pub struct SingleThread {
    sender: Sender<Box<ExecBox>>,
}

impl SingleThread {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            for exec in rx {
                exec.exec();
            }
        });

        SingleThread {
            sender: tx
        }
    }
}

impl Executor for SingleThread {
    fn execute(&self, exec: Box<ExecBox>) {
        self.sender.send(exec);
    }
}

pub struct SyncExecutor;

impl Executor for SyncExecutor {
    fn execute(&self, exec: Box<ExecBox>) {
        exec.exec();
    }
}