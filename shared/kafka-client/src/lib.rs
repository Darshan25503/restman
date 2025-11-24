use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::consumer::{Consumer, StreamConsumer};
use serde::Serialize;
use std::time::Duration;

pub mod producer;
pub mod consumer;

pub use producer::KafkaProducer;
pub use consumer::KafkaConsumer;

/// Create a Kafka producer
pub fn create_producer(brokers: &str) -> Result<FutureProducer, rdkafka::error::KafkaError> {
    tracing::info!("Creating Kafka producer");
    
    ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("message.timeout.ms", "5000")
        .set("compression.type", "snappy")
        .create()
}

/// Create a Kafka consumer
pub fn create_consumer(
    brokers: &str,
    group_id: &str,
    topics: &[&str],
) -> Result<StreamConsumer, rdkafka::error::KafkaError> {
    tracing::info!("Creating Kafka consumer for group: {}", group_id);
    
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("group.id", group_id)
        .set("enable.auto.commit", "true")
        .set("auto.offset.reset", "earliest")
        .create()?;

    consumer.subscribe(topics)?;
    
    Ok(consumer)
}

/// Publish a message to Kafka
pub async fn publish_message<T: Serialize>(
    producer: &FutureProducer,
    topic: &str,
    key: Option<&str>,
    payload: &T,
) -> Result<(), anyhow::Error> {
    let payload_json = serde_json::to_string(payload)?;
    
    let mut record = FutureRecord::to(topic).payload(&payload_json);
    
    if let Some(k) = key {
        record = record.key(k);
    }
    
    producer
        .send(record, Duration::from_secs(5))
        .await
        .map_err(|(err, _)| anyhow::anyhow!("Failed to send message: {}", err))?;
    
    tracing::debug!("Published message to topic: {}", topic);
    
    Ok(())
}

