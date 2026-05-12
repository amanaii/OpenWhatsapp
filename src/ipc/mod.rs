//! App event bus.

use tokio::sync::broadcast;

pub(crate) mod events;

pub(crate) use events::AppEvent;

/// Default app event channel capacity.
pub(crate) const DEFAULT_EVENT_CAPACITY: usize = 256;

/// Tokio broadcast-backed app event bus.
#[derive(Clone, Debug)]
pub(crate) struct EventBus {
    sender: broadcast::Sender<AppEvent>,
}

impl EventBus {
    /// Creates a new event bus with the given channel capacity.
    pub(crate) fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Emits an app event and returns the active subscriber count.
    pub(crate) fn emit(&self, event: AppEvent) -> usize {
        self.sender.send(event).unwrap_or_default()
    }

    /// Subscribes to app events.
    pub(crate) fn subscribe(&self) -> broadcast::Receiver<AppEvent> {
        self.sender.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_bus_broadcasts_to_subscriber() {
        let bus = EventBus::new(8);
        let mut receiver = bus.subscribe();

        assert_eq!(bus.emit(AppEvent::AppReady), 1);
        assert_eq!(receiver.try_recv().unwrap(), AppEvent::AppReady);
    }

    #[test]
    fn event_bus_ignores_missing_subscribers() {
        let bus = EventBus::new(8);

        assert_eq!(bus.emit(AppEvent::AppReady), 0);
    }
}
