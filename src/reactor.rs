use crate::epoll::{Epoll, EpollEventType};
use crate::{info, io, mem, BTreeMap, Mutex, RawFd, Waker};

use lazy_static::lazy_static;

lazy_static! {
    pub(crate) static ref REACTOR: Reactor = {
        // Start reactor main loop
        std::thread::spawn(move || {
            reactor_main_loop()
        });

        Reactor {
            epoll: Epoll::new().expect("failed to create epoll"),
            wakers: Mutex::new(BTreeMap::new())
        }
    };
}

pub(crate) struct Reactor {
    pub epoll: Epoll,
    pub wakers: Mutex<BTreeMap<RawFd, Waker>>,
}

impl Reactor {
    pub(crate) fn add_event(&self, fd: RawFd, op: EpollEventType, waker: Waker) -> io::Result<()> {
        info!("(Reactor) add event: {}", fd);
        self.epoll.add_event(fd, op)?;
        self.wakers.lock().unwrap().insert(fd, waker);
        Ok(())
    }
}

fn reactor_main_loop() -> io::Result<()> {
    info!("Start reactor main loop");
    let max_event = 32;
    let event: libc::epoll_event = unsafe { mem::zeroed() };
    let mut events = vec![event; max_event];
    let reactor = &REACTOR;

    loop {
        let nfd = reactor.epoll.wait(&mut events)?;
        info!("(Reactor) wake up. nfd = {}", nfd);

        #[allow(clippy::needless_range_loop)]
        for i in 0..nfd {
            let fd = events[i].u64 as RawFd;
            if let Some(waker) = reactor.wakers.lock().unwrap().remove(&fd) {
                info!("(Reactor) delete event: {}", fd);
                reactor.epoll.del_event(fd)?;
                waker.wake();
            }
            // let waker = reactor
            //     .wakers
            //     .lock()
            //     .unwrap()
            //     .remove(&fd)
            //     .unwrap_or_else(|| panic!("not found fd {}", fd));
            // info!("(Reactor) delete event: {}", fd);
            // reactor.epoll.del_event(fd)?;
            // waker.wake();
        }
    }
}
