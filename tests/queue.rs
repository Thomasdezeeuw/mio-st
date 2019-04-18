use std::time::Duration;

use log::error;

use mio_st::event::{self, Capacity, Ready, Source};
use mio_st::{Event, Queue};

mod util;

use self::util::{init, next_event_available, EventsCapacity};

#[test]
fn queue() {
    init();
    let mut queue = Queue::new();
    let mut events = Vec::new();

    assert_eq!(next_event_available(&mut queue), None);

    // Single event.
    let event = Event::new(event::Id(0), Ready::READABLE);
    queue.add(event);
    assert_eq!(next_event_available(&mut queue), Some(Duration::from_millis(0)));
    expect_events(&mut queue, &mut events, vec![event]);

    // Multiple events.
    queue.add(Event::new(event::Id(0), Ready::READABLE));
    queue.add(Event::new(event::Id(0), Ready::WRITABLE));
    queue.add(Event::new(event::Id(0), Ready::READABLE | Ready::WRITABLE));
    queue.add(Event::new(event::Id(1), Ready::READABLE | Ready::WRITABLE | Ready::ERROR));
    assert_eq!(next_event_available(&mut queue), Some(Duration::from_millis(0)));
    expect_events(&mut queue, &mut events, vec![
        Event::new(event::Id(0), Ready::READABLE),
        Event::new(event::Id(0), Ready::WRITABLE),
        Event::new(event::Id(0), Ready::READABLE | Ready::WRITABLE),
        Event::new(event::Id(1), Ready::READABLE | Ready::WRITABLE | Ready::ERROR),
    ]);
}

#[test]
fn queue_events_capacity() {
    init();
    let mut queue = Queue::new();

    let event = Event::new(event::Id(0), Ready::READABLE);
    queue.add(event);
    queue.add(event);

    let mut events = EventsCapacity(Capacity::Limited(0), 0);
    Source::<_, ()>::poll(&mut queue, &mut events).unwrap();
    assert_eq!(events.1, 0); // Shouldn't have grow.

    let mut events = EventsCapacity(Capacity::Limited(1), 0);
    Source::<_, ()>::poll(&mut queue, &mut events).unwrap();
    assert_eq!(events.1, 1);

    let mut events = EventsCapacity(Capacity::Limited(1), 0);
    Source::<_, ()>::poll(&mut queue, &mut events).unwrap();
    assert_eq!(events.1, 1);

    let mut events = EventsCapacity(Capacity::Limited(100), 0);
    queue.add(event);
    queue.add(event);
    Source::<_, ()>::poll(&mut queue, &mut events).unwrap();
    assert_eq!(events.1, 2);

    let mut events = EventsCapacity(Capacity::Growable, 0);
    queue.add(event);
    queue.add(event);
    Source::<_, ()>::poll(&mut queue, &mut events).unwrap();
    assert_eq!(events.1, 2);
}

/// Poll `Queue` for events.
fn expect_events(queue: &mut Queue, events: &mut Vec<Event>, mut expected: Vec<Event>) {
    events.clear();
    Source::<_, ()>::poll(queue, events)
        .expect("unable to poll user space queue");

    for event in events.drain(..) {
        let index = expected.iter()
            .position(|expected| {
                event.id() == expected.id() &&
                event.readiness().contains(expected.readiness())
            });

        if let Some(index) = index {
            expected.swap_remove(index);
        } else {
            // Must accept sporadic events.
            error!("got unexpected event: {:?}", event);
        }
    }

    assert!(expected.is_empty(), "the following expected events were not found: {:?}", expected);
}
