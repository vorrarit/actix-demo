use std::error::Error;

use actix_web::{http::{header::ContentType, StatusCode}, HttpResponse, ResponseError};

#[derive(thiserror::Error, Debug)]
pub enum ActixDemoError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error)
}

impl ResponseError for ActixDemoError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(format!(r#"{{ "message": "{}", "source": "{}" }}"#, self.to_string(), self.source().map_or(String::from(""), |s| s.to_string())))
    }
}
