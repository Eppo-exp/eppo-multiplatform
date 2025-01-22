use crate::event_ingestion::event::Event;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct QueuedEvent {
    pub event: Event,
    pub attempts: u32,
}

impl QueuedEvent {
    pub fn new(event: Event) -> Self {
        QueuedEvent { event, attempts: 0 }
    }

    /// Creates a new QueuedEvent from a failed QueuedEvent, incrementing the attempts counter.
    pub fn new_from_failed(queued_event: QueuedEvent) -> Self {
        QueuedEvent {
            event: queued_event.event,
            attempts: queued_event.attempts + 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::event_ingestion::event::Event;
    use crate::event_ingestion::queued_event::QueuedEvent;
    use crate::timestamp::now;

    #[test]
    fn test_new() {
        let event = Event {
            uuid: uuid::Uuid::new_v4(),
            timestamp: now(),
            event_type: "test".to_string(),
            payload: serde_json::json!({"key": "value"}),
        };
        let queued_event = QueuedEvent::new(event.clone());
        assert_eq!(queued_event.event, event);
        assert_eq!(queued_event.attempts, 0);
        assert_eq!(queued_event.event.event_type, "test");
    }
}
