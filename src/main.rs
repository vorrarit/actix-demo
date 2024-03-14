use std::error::Error;

use actix_demo::run;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    run().await
}