use std::sync::{Arc, Mutex};

use futures::Poll;
use futures::Async::{NotReady, Ready};
use futures::prelude::Future;
use futures::task;

use self::Inner::*;

#[derive(Clone, Debug)]
pub struct BlackHole(Arc<Mutex<Inner>>);

#[derive(Debug)]
enum Inner {
    Released,
    Wait(Vec<task::Task>),
}

impl BlackHole {
    pub fn new() -> BlackHole {
        BlackHole(Arc::new(Mutex::new(Wait(Vec::new()))))
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
}

impl Future for BlackHole {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<(), ()> {
        match *self.0.lock().unwrap() {
            Released => Ok(Ready(())),
            Wait(ref mut tasks) => {
                tasks.push(task::current());
                Ok(NotReady)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::{channel, Sender};
    use std::thread::sleep;
    use std::time::Duration;

    use futures::prelude::*;
    use futures_cpupool::CpuPool;

    use black_hole::*;

    #[test]
    fn black_hole_new() {
        BlackHole::new();
    }

    #[test]
    fn black_hole_release() {
        BlackHole::new().release();
    }

    #[async]
    fn send(s: Sender<i32>, b: BlackHole) -> Result<(), ()> {
        s.send(1).unwrap();
        await!(b)?;
        s.send(3).unwrap();
        Ok(())
    }

    #[async]
    fn release(s: Sender<i32>, b: BlackHole) -> Result<(), ()> {
        s.send(2).unwrap();
        b.release();
        Ok(())
    }

    #[test]
    fn black_hole_wait() {
        let p = CpuPool::new_num_cpus();

        let b = BlackHole::new();
        let (s, r) = channel();

        assert!(r.try_recv().is_err());

        let f1 = p.spawn(send(s.clone(), b.clone()));

        sleep(Duration::from_millis(100));
        assert_eq!(r.recv().unwrap(), 1);
        assert!(r.try_recv().is_err());

        let f2 = p.spawn(release(s.clone(), b.clone()));

        sleep(Duration::from_millis(100));
        assert_eq!(r.recv().unwrap(), 2);

        sleep(Duration::from_millis(100));
        assert_eq!(r.recv().unwrap(), 3);
        assert!(r.try_recv().is_err());

        assert!(f1.wait().is_ok());
        assert!(f2.wait().is_ok());
    }
}
