use static_vector::{Vec, vec};

#[derive(Debug, PartialEq, Clone, Copy)]
enum Event {
    Start,
    Load,
    Run,
    Pause,
    Resume,
    Stop,
    Exit,
}

struct EventHistory<const N: usize> {
    events: Vec<Event, N>,
}

impl<const N: usize> EventHistory<N> {
    const fn new() -> Self {
        Self { events: vec![Event; N] }
    }

    fn insert(&mut self, event: Event) {
        if self.events.is_full() {
            // Remove the oldest event (FIFO)
            let len = self.events.len();
            self.events.as_mut_slice().copy_within(1..len, 0);

            // Can ignore the error here since we are guaranteed to have at least one element
            // because the vector is full.
            let _ = self.events.pop();
        }

        // Can ignore the error here since we are guaranteed to have space after popping
        // if the vector was full.
        let _ = self.events.push(event);
    }

    const fn get_events(&self) -> &[Event] {
        self.events.as_slice()
    }
}

// Example of using static_vector to maintain a history of events
fn main() {
    const HISTORY_SIZE: usize = 5;
    let mut history = EventHistory::<HISTORY_SIZE>::new();

    let events = [
        Event::Start,
        Event::Load,
        Event::Run,
        Event::Pause,
        Event::Resume,
        Event::Stop,
        Event::Exit,
    ];
    for &event in &events {
        history.insert(event);
    }

    assert_eq!(
        history.get_events(),
        &[Event::Run, Event::Pause, Event::Resume, Event::Stop, Event::Exit],
        "Expected history to contain the last 5 events"
    );
}
