use std::env;

use actix_web::{get, http::Error, post, web, App, HttpMessage, HttpRequest, HttpResponse, HttpServer, Responder};
use anyhow::anyhow;
use opentelemetry::{global, trace::{FutureExt, TraceContextExt}, Context};
use opentelemetry_auto_span::auto_span;
use opentelemetry_http::HeaderInjector;
use reqwest::header::HeaderMap;
use serde::Deserialize;
use sqlx::{pool, Sqlite, SqlitePool};
use tracing::{debug, info, Span};
use tracing_actix_web::RequestId;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use utils::configuration::Configuration;

mod utils;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[tracing::instrument]
#[post("/echo")]
async fn echo(req: HttpRequest, req_body: String) -> impl Responder {
    let mut res = HttpResponse::Ok();

    info!("### HEADERS ###");
    req.headers().into_iter().for_each(| (k, v) | {
        info!("{} {}", k.as_str(), v.to_str().unwrap());
        res.append_header(("req_".to_owned() + k.as_str(), v.to_str().unwrap()));
    });

    info!("### BODY ###");
    info!(req_body);

    res.body(req_body)
}

#[tracing::instrument]
#[post("/serviceb")]
async fn serviceb(conf: web::Data<Configuration>, req: HttpRequest, req_body: String) -> Result<HttpResponse, actix_web::Error> {
    let req_id = req.extensions().get::<RequestId>().unwrap();
    let mut headers = HeaderMap::new();

    info!("### HEADERS ###");
    for header in req.headers().into_iter() {
        info!("{:?} = {:?}", header.0, header.1);
        headers.append(header.0, header.1.clone());
    }
    info!("### BODY ###");
    info!(req_body);

    // req.headers().into_iter().for_each(| (k, v) | {
    //     info!("{} {}", k.as_str(), v.to_str().unwrap());
    //     headers.append(k.as_str(), v.to_str().unwrap().parse().unwrap());
    // });

    let serviceb_url = &conf.service_b.url;
    let client = reqwest::Client::new();
    let mut rqw_request = client.post(serviceb_url)
        .headers(headers)
        .body(req_body)
        .build().map_err(|e| {
            utils::error::Error(anyhow!(e))
        })?;

    let cx = Span::current().context();
    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&cx, &mut HeaderInjector(rqw_request.headers_mut()))
    });

   let rqw_response = client.execute(rqw_request)
        .await.map_err(|e| {
            utils::error::Error(anyhow!(e))
        })?;
    

    let mut res = HttpResponse::Ok();
    for header in rqw_response.headers().into_iter() {
        res.append_header((header.0, header.1));
    }
    let body = rqw_response.text()
        .await.map_err(|e| {
            utils::error::Error(anyhow!(e))
        })?;

    Ok(res.body(body))
}

#[tracing::instrument]
pub async fn manual_hello() -> impl Responder {
    info!("manual_hello");
    HttpResponse::Ok().body("Hey there!")
}

#[derive(Deserialize, Debug)]
struct TodoItem {
    description: String,
    done: bool
}

#[tracing::instrument]
#[post("/todo")]
pub async fn todo_add(pool: web::Data<SqlitePool>, todo_item: web::Json<TodoItem>) -> Result<HttpResponse, actix_web::Error> {
    let mut conn = pool.acquire()
        .await
        .map_err(|e| {
            utils::error::Error(anyhow!(e))
        })?;
    
    let result = sqlx::query("insert into todos (description) values (?1)")
        .bind(todo_item.description.clone())
        .execute(&mut *conn)
        .await.map_err(|e| {
            utils::error::Error(anyhow!(e))
        })?;
    
    Ok(HttpResponse::Ok().body(result.last_insert_rowid().to_string()))
}

pub async fn run()  -> Result<(), Box<dyn std::error::Error>> {
    let conf = utils::configuration::Configuration::new()?;
    if conf.application.otel.enable {
        let opentelemetry = utils::opentelemetry::init_layer(&conf.application.otel.grpc_url, &conf.application.name)?;
        utils::tracibility::init(Some(opentelemetry))?;
    } else {
        utils::tracibility::init(None)?;
    }
    let (metrics_handler, meter_provider) = utils::prometheus::init()?;
    
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;
    let port = conf.application.port;
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(conf.clone()))
            .app_data(web::Data::new(pool.clone()))
            .wrap(tracing_actix_web::TracingLogger::default())
            .wrap(actix_web_opentelemetry::RequestMetrics::default())
            .service(hello)
            .service(echo)
            .service(serviceb)
            .service(todo_add)
            .route("/hey", web::get().to(manual_hello))
            .route("/metrics", web::get().to(metrics_handler.clone()))
    })
    .bind(("0.0.0.0", port))?
    .run().await.map_err(|err| format!("Error {}", err).into())

    // global::shutdown_tracer_provider();
    // meter_provider.shutdown();
}