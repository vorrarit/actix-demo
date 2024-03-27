use opentelemetry::{global::{self, ObjectSafeSpan}, propagation::Extractor, trace::Tracer, Key, KeyValue, StringValue};
use rdkafka::{config::RDKafkaLogLevel, consumer::{BaseConsumer, CommitMode, Consumer, ConsumerContext, Rebalance, StreamConsumer}, error::KafkaResult, message::{BorrowedHeaders, Headers}, ClientConfig, ClientContext, Message, TopicPartitionList};
use tracing::{info, warn};

pub struct CustomContext;

impl ClientContext for CustomContext {}

impl ConsumerContext for CustomContext {
    fn pre_rebalance(&self, rebalance: &Rebalance) {
        info!("Pre rebalance {:?}", rebalance);
    }

    fn post_rebalance(&self, rebalance: &Rebalance) {
        info!("Post rebalance {:?}", rebalance);
    }

    fn commit_callback(&self, result: KafkaResult<()>, _offsets: &TopicPartitionList) {
        info!("Committing offsets: {:?}", result);
    }
}

type LoggingConsumer = StreamConsumer<CustomContext>;

pub async fn consume_and_print(brokers: &str, group_id: &str, topic: &str) {
    let context = CustomContext;

    let consumer: LoggingConsumer = ClientConfig::new()
        .set("group.id", group_id)
        .set("bootstrap.servers", brokers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        //.set("statistics.interval.ms", "30000")
        //.set("auto.offset.reset", "smallest")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create_with_context(context)
        .expect("Consumer creation failed");

    consumer
        .subscribe(vec![topic].as_slice())
        .expect("Can't subscribe to specified topics");

    loop {
        match consumer.recv().await {
            Err(e) => warn!("Kafka error: {}", e),
            Ok(m) => {
                let payload = match m.payload_view::<str>() {
                    None => "",
                    Some(Ok(s)) => s,
                    Some(Err(e)) => {
                        warn!("Error while deserializing message payload: {:?}", e);
                        ""
                    }
                };
                info!("key: '{:?}', payload: '{}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                      m.key(), payload, m.topic(), m.partition(), m.offset(), m.timestamp());
                if let Some(headers) = m.headers() {
                    for header in headers.iter() {
                        info!("  Header {:#?}: {:?}", header.key, header.value);
                    }

                    //(1)
                    let context = global::get_text_map_propagator(|propagator| {
                        propagator.extract(&HeaderExtractor(&headers))
                    });

                    //(2)
                    let mut span =
                        global::tracer("consumer").start_with_context("consume_payload", &context);
                    span.set_attribute(KeyValue { key: Key::new("payload"), value: opentelemetry::Value::String(StringValue::from(payload.to_string())) });
                    span.end();
                }
                consumer.commit_message(&m, CommitMode::Async).unwrap();
            }
        };
    }
}

pub struct HeaderExtractor<'a>(pub &'a BorrowedHeaders);

    impl<'a> Extractor for HeaderExtractor<'a> {
        fn get(&self, key: &str) -> Option<&str> {
            for i in 0..self.0.count() {
                if let Ok(val) = self.0.get_as::<str>(i) {
                    if val.key == key {
                        return val.value
                    }
                }
            }
            None
        }

        fn keys(&self) -> Vec<&str> {
            self.0.iter().map(|kv| kv.key).collect::<Vec<_>>()
        }
    }