use anyhow::Context;
use rdkafka::{producer::FutureProducer, ClientConfig};

pub fn init_kafka_producer(broker: &str) -> Result<FutureProducer, anyhow::Error> {
    let p: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", broker)
        .set("message.timeout.ms", "5000")
        .create()
        .context("Error creating producer")?;

    Ok(p)
}