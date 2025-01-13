use crate::events::event::Event;

#[derive(Debug, Clone, PartialEq)]
pub enum QueuedEventStatus {
    PENDING,
    RETRY,
    FAILED,
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueuedEvent {
    pub event: Event,
    pub attempts: u8,
    pub status: QueuedEventStatus,
}

impl QueuedEvent {
    pub fn new(event: Event) -> Self {
        QueuedEvent {
            event,
            attempts: 0,
            status: QueuedEventStatus::PENDING,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::events::event::Event;
    use crate::events::queued_event::{QueuedEvent, QueuedEventStatus};
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
        assert_eq!(queued_event.status, QueuedEventStatus::PENDING);
    }
}
