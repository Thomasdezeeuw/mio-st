use mio::*;
use std::time::Duration;

#[test]
fn test_poll_closes_fd() {
    for _ in 0..2000 {
        let mut poll = Poll::new().unwrap();
        let mut events = Events::with_capacity(4);
        let (mut registration, set_readiness) = Registration::new2();

        poll.register(&mut registration, Token(0), Ready::READABLE, PollOpt::EDGE).unwrap();
        poll.poll(&mut events, Some(Duration::from_millis(0))).unwrap();

        drop(poll);
        drop(set_readiness);
        drop(registration);
    }
}
