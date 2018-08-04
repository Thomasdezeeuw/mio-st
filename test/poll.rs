use std::time::Instant;

use mio_st::event::{Event, EventedId, Ready};
use mio_st::poll::PollOption;
use mio_st::registration::Registration;

use init_with_poll;

// TODO: add tests for both TcpStream and TcpListener:
// reregistering and
// deregistering.

// Keep in sync the actual size in `Events`.
const EVENTS_CAP: usize = 256;

#[test]
fn polling_userspace_dont_expand_events() {
    let (mut poll, mut events) = init_with_poll();

    let (mut registration, mut notifier) = Registration::new();
    poll.register(&mut registration, EventedId(0), Ready::READABLE, PollOption::Edge).unwrap();

    for _ in 0..EVENTS_CAP + 1 {
        notifier.notify(Ready::READABLE).unwrap();
    }

    let mut check = |length| {
        poll.poll(&mut events, None).unwrap();
        assert_eq!(events.len(), length);
        for event in &mut events {
            assert_eq!(event, Event::new(EventedId(0), Ready::READABLE));
        }
    };

    check(EVENTS_CAP);
    check(1);
}

#[test]
fn polling_deadlines_dont_expand_events() {
    let (mut poll, mut events) = init_with_poll();

    let now = Instant::now();
    for _ in 0..EVENTS_CAP + 1 {
        poll.add_deadline(EventedId(0), now).unwrap();
    }

    let mut check = |length| {
        poll.poll(&mut events, None).unwrap();
        assert_eq!(events.len(), length);
        for event in &mut events {
            assert_eq!(event, Event::new(EventedId(0), Ready::TIMER));
        }
    };

    check(EVENTS_CAP);
    check(1);
}
