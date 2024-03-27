use serde::Deserialize;
use config::{Config, ConfigError, File};

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct DemoConsumer {
    pub enable: bool,
    pub group_id: Option<String>,
    pub topic: Option<String>
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Kafka {
    pub broker: String
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct ServiceB {
    pub url: String
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Database {
    pub url: String
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Otel {
    pub enable: bool,
    pub grpc_url: String
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Application {
    pub name: String,
    pub port: u16,
    pub otel: Otel,
    pub database: Database,
    pub kafka: Option<Kafka>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Configuration {
    pub application: Application,
    pub service_b: ServiceB,
    pub demo_consumer: Option<DemoConsumer>
}

impl Configuration {
    #[warn(dead_code)]
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name("config"))
            .build()?;

        s.try_deserialize()
    }
}