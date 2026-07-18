use super::*; 
use std::path::Path;
use tokio::time::Duration;
use serde::{Deserialize, Serialize};
use bytes::Bytes;

pub struct MockBroker;
impl MockBroker {
    pub fn attach_probe<F>(&self, _f: F) {}
    pub async fn publish(&self, _topic: &std::sync::Arc<str>, _packet: hotaru_mqtt::PublishPacket) {}
}
const BROKER: MockBroker = MockBroker;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct FixtureEvent {
    pub t: u64,
    pub dir: String,
    pub topic: String,
    pub payload: serde_json::Value,
}

fn canonicalize(events: &[FixtureEvent]) -> String {
    let mut sorted_events = events.to_vec();
    sorted_events.sort_by(|a, b| a.topic.cmp(&b.topic)); 
    serde_json::to_string(&sorted_events).unwrap_or_default()
}

async fn run_fixture(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file_content = tokio::fs::read_to_string(path).await?;
    let mut events: Vec<FixtureEvent> = vec![];
    for line in file_content.lines() {
        if line.trim().is_empty() { continue; }
        let ev: FixtureEvent = serde_json::from_str(line)?;
        events.push(ev);
    }

    let mut seen_out: Vec<FixtureEvent> = vec![];
    
    let _probe = BROKER.attach_probe(|_ev: hotaru_mqtt::PublishPacket| {
    });

    for ev in events.iter().filter(|e| e.dir == "in") {
        let payload_bytes = Bytes::from(serde_json::to_vec(&ev.payload)?);
        let packet = hotaru_mqtt::PublishPacket {
            topic: std::sync::Arc::from(ev.topic.as_str()),
            payload: payload_bytes,
            qos: hotaru_mqtt::QoS::AtLeastOnce,
            retain: false,
            dup: false,
            packet_id: None,
        };
        BROKER.publish(&std::sync::Arc::from("test_fixture"), packet).await;
        tokio::time::sleep(Duration::from_millis(ev.t)).await;
    }

    let expected: Vec<_> = events.into_iter().filter(|e| e.dir == "out").collect();

    assert_eq!(
        canonicalize(&seen_out), 
        canonicalize(&expected), 
        "Fixture failed: Output did not strictly match expected Appendix C output"
    );

    Ok(())
}

#[tokio::test]
async fn test_fixture_1_happy_path() {
    run_fixture(Path::new("fixtures/1_happy_path.jsonl")).await.unwrap();
}