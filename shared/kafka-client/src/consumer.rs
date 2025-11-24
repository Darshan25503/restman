use rdkafka::consumer::StreamConsumer;

/// Wrapper for Kafka consumer
pub struct KafkaConsumer {
    consumer: StreamConsumer,
}

impl KafkaConsumer {
    pub fn new(consumer: StreamConsumer) -> Self {
        Self { consumer }
    }

    pub fn inner(&self) -> &StreamConsumer {
        &self.consumer
    }
}

