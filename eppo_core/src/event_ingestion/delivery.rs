use tokio::sync::mpsc;

use super::{BatchedMessage, Event};

pub(super) struct DeliveryStatus {
    success: Vec<Event>,
    failure: Vec<Event>,
}

pub(super) async fn delivery(
    mut uplink: mpsc::Receiver<BatchedMessage<Event>>,
    delivery_status: mpsc::Sender<DeliveryStatus>,
    // TODO: configuration
) {
}
