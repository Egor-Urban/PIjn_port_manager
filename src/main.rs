/*
Port manager microservice for PIjN protocol project
Developer: Urban Egor
Version: 5.5.20 r
*/

use actix_web::{get, web, App, HttpServer, HttpResponse, Responder, middleware::Logger, HttpRequest};
use serde::Deserialize;
use chrono::Local;
use tracing::{info, warn};
use tracing_subscriber;

mod port_manager;

#[derive(Deserialize)]
struct PathParams {
    service_name: String,
}

// ------------------ Handlers --------------------

#[get("/getport/{service_name}")]
async fn get_port_handler(req: HttpRequest, path: web::Path<PathParams>) -> impl Responder {
    let client_addr = req.peer_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let service_name = &path.service_name;

    match port_manager::resolve_service_port(service_name) {
        Ok(port) => {
            info!(target: "get_port_handler", "Client {} got port {} for '{}'", client_addr, port, service_name);
            HttpResponse::Ok().json(serde_json::json!({ "success": true, "data": port }))
        }
        Err(err_msg) => {
            warn!(target: "get_port_handler", "Client {} — {}", client_addr, err_msg);
            HttpResponse::NotFound().json(serde_json::json!({ "success": false, "error": err_msg }))
        }
    }
}

#[get("/status")]
async fn get_module_status(req: HttpRequest) -> impl Responder {
    let client_addr = req.peer_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    info!(target: "status_handler", "Client {} requested status", client_addr);
    HttpResponse::Ok().json(serde_json::json!({ "success": true, "data": null }))
}

// ------------------ Main --------------------

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_tracing();

    let port = 1030;
    let ip = "127.0.0.1";

    info!(target: "main", "Starting port manager microservice on {}:{}", ip, port);
    info!(target: "main", "Version: 4.4.14 r");

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(get_port_handler)
            .service(get_module_status)
    })
    .workers(4)
    .bind((ip, port))?
    .run()
    .await
}

fn init_tracing() {
    let date = Local::now().format("%d_%m_%Y").to_string();
    let log_path = format!("./logs/port_manager_microservice_{}.log", date);
    std::fs::create_dir_all("./logs").ok();

    tracing_subscriber::fmt()
        .with_target(true)
        .with_writer(std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .expect("Cannot open log file"))
        .with_thread_names(true)
        .with_ansi(false)
        .init();
}
