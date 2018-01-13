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

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::mpsc::channel;
    use std::thread;
    use std::time::Duration;

    use black_hole::*;

    #[test]
    fn black_hole_new() {
        BlackHole::new();
    }

    #[test]
    fn black_hole_release() {
        BlackHole::new().release();
    }

    #[test]
    fn black_hole_wait() {
        let b = Arc::new(BlackHole::new());
        let (s, r) = channel();

        assert!(r.try_recv().is_err());

        let ss = s.clone();
        let bb = b.clone();
        thread::spawn(move || {
            ss.send(1).unwrap();
            bb.wait();
        });

        thread::sleep(Duration::from_millis(100));

        assert_eq!(r.recv().unwrap(), 1);
        assert!(r.try_recv().is_err());

        thread::spawn(move || {
            s.send(2).unwrap();
            b.release();
        });

        thread::sleep(Duration::from_millis(100));

        assert_eq!(r.recv().unwrap(), 2);
        assert!(r.try_recv().is_err());
    }
}
