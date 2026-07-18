use std::sync::Arc;
use bytes::Bytes;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum QoS {
    AtMostOnce,
    AtLeastOnce,
    ExactlyOnce,
}

#[derive(Clone, Debug)]
pub struct PublishPacket {
    pub topic: Arc<str>,
    pub payload: Bytes,
    pub qos: QoS,
    pub retain: bool,
    pub dup: bool,
    pub packet_id: Option<u16>,
}