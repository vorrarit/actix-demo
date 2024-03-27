use std::time::Duration;

use actix_web::{body, http::StatusCode, post, web, HttpMessage, HttpRequest, HttpResponse};
use anyhow::Context;
use opentelemetry::{global, propagation::Injector, trace::TraceId};
use rdkafka::{message::{Header, Headers, OwnedHeaders}, producer::{FutureProducer, FutureRecord}, ClientConfig};
use serde::Deserialize;
use tracing::{error, info, Span};
use tracing_actix_web::RequestId;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use uuid::{NoContext, Timestamp, Uuid};

use crate::utils;

#[tracing::instrument(skip(producer))]
#[post("/publish")]
pub async fn publish(req: HttpRequest, producer: web::Data<FutureProducer>, message: web::Json<Message>) -> Result<HttpResponse, utils::error::ActixDemoError> {
    // let req_id = req.extensions().get::<RequestId>().unwrap().clone();
    // let clone_req_id = req_id.to_string();
    let uuid = Uuid::new_v7(Timestamp::now(NoContext));

    let cx = Span::current().context();
    let mut headers = OwnedHeaders::new().insert(Header { key: "header_key", value: Some("header_value") });
    //(1)
    global::get_text_map_propagator(|propagator| {
        //(2)
        propagator.inject_context(&cx, &mut HeaderInjector(&mut headers))
    });

    let delivery_status = producer
        .send(
            FutureRecord::to("actixweb_demo")
                .payload(&format!("Message {}", message.text))
                .key(&format!("{}", uuid))
                .headers(headers),
            Duration::from_secs(0),
        )
        .await;
    
    match delivery_status {
        Ok((i, j)) => {
            info!("delivery status i={}, j={}", i, j);
            Ok(HttpResponse::Ok().body("OK"))
        },
        Err((k_err, own_message)) => {
            error!("delivery error k_err={:?}, own_message={:?}", k_err, own_message);
            Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).body("OK"))
        }
    }

    
}

#[derive(Deserialize, Debug)]
struct Message {
    text: String
}

pub struct HeaderInjector<'a>(pub &'a mut OwnedHeaders);

impl <'a>Injector for HeaderInjector<'a> {
    fn set(&mut self, key: &str, value: String) {
        let mut new = OwnedHeaders::new().insert(rdkafka::message::Header {
            key,
            value: Some(&value),
        });

        for header in self.0.iter() {
            let s = String::from_utf8(header.value.unwrap().to_vec()).unwrap();
            new = new.insert(rdkafka::message::Header { key: header.key, value: Some(&s) });
        }

        self.0.clone_from(&new);
    }
}