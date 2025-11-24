use rdkafka::producer::FutureProducer;
use serde::Serialize;

/// Wrapper for Kafka producer
pub struct KafkaProducer {
    producer: FutureProducer,
}

impl KafkaProducer {
    pub fn new(producer: FutureProducer) -> Self {
        Self { producer }
    }

    pub async fn publish<T: Serialize>(
        &self,
        topic: &str,
        key: Option<&str>,
        payload: &T,
    ) -> Result<(), anyhow::Error> {
        crate::publish_message(&self.producer, topic, key, payload).await
    }
}

