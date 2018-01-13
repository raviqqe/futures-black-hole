use futures::Async;
use std::sync::Mutex;

use futures::task;
use self::Inner::*;

#[derive(Debug)]
pub struct BlackHole(Mutex<Inner>);

#[derive(Debug)]
enum Inner {
    Released,
    Wait(Vec<task::Task>),
}

impl BlackHole {
    pub fn new() -> BlackHole {
        BlackHole(Mutex::new(Wait(Vec::new())))
    }

    pub fn release(&self) {
        let mut inner = self.0.lock().unwrap();

        match *inner {
            Released => panic!("BlackHole is released twice"),
            Wait(ref tasks) => for task in tasks {
                task.notify();
            },
        }

        *inner = Released;
    }

    pub fn wait(&self) -> Async<()> {
        match *self.0.lock().unwrap() {
            Released => Async::Ready(()),
            Wait(ref mut tasks) => {
                tasks.push(task::current());
                Async::NotReady
            }
        }
    }
}
