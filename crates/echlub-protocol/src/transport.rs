use crate::delivery::DeliverySemantics;
use crate::flow::Flow;

pub trait Transport {
    fn send(
        &mut self,
        flow: Flow,
        payload: &[u8],
        delivery: DeliverySemantics,
    ) -> Result<(), TransportError>;
}

#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("transport unavailable")]
    Unavailable,
}
