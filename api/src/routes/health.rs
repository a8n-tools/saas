//! Health check and status endpoints

use actix_web::{get, web, HttpResponse};
use serde::Serialize;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

#[derive(Serialize)]
struct StatusResponse {
    service: &'static str,
    version: &'static str,
    commit: &'static str,
}

/// Root status endpoint at /
#[get("/")]
pub async fn root_status() -> HttpResponse {
    HttpResponse::Ok().json(StatusResponse {
        service: "a8n-api",
        version: env!("CARGO_PKG_VERSION"),
        commit: env!("GIT_COMMIT"),
    })
}

/// Health check endpoint at /health
#[get("/health")]
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Health check endpoint at /v1/health
#[get("/health")]
async fn health_check_v1() -> HttpResponse {
    HttpResponse::Ok().json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Configure health routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(health_check_v1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    #[actix_rt::test]
    async fn test_health_check() {
        let app = test::init_service(
            App::new().service(health_check)
        ).await;

        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }
}
