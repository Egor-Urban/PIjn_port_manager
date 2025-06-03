/*
Port manager microservice for PIjN protocol project
Developer: Urban Egor
Version: 6.5.21 r
*/



use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tokio::time::Duration;
use tracing::{info};

mod status;
mod utils;
mod port_manager;

use status::get_status;
use utils::{init_tracing, load_config};
use port_manager::resolve_service_port;



#[derive(Serialize, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: T,
}


#[derive(Deserialize)]
struct PathParams {
    service_name: String,
}



#[get("/getport/{service_name}")]
async fn get_port_handler(req: HttpRequest, path: web::Path<PathParams>) -> impl Responder {
    let client_addr = req.peer_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let service_name = &path.service_name;

    match resolve_service_port(service_name) {
        Ok(port) => {
            info!(target: "get_port_handler", "Client {} got port {} for '{}'", client_addr, port, service_name);
            HttpResponse::Ok().json(serde_json::json!({ "success": true, "data": port }))
        }
        Err(err_msg) => {
            tracing::warn!(target: "get_port_handler", "Client {} — {}", client_addr, err_msg);
            HttpResponse::NotFound().json(serde_json::json!({ "success": false, "error": err_msg }))
        }
    }
}


#[get("/status")]
async fn status_handler(start: web::Data<Instant>, req: HttpRequest) -> impl Responder {
    let client_addr = req
        .peer_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    let status_json = get_status(*start.get_ref());
    let status = serde_json::json!({ "success": true, "data": status_json });

    info!(target: "status_handler", "Client {} requested status: {}", client_addr, status);
    HttpResponse::Ok().json(status)
}


#[get("/stop")]
async fn stop_handler() -> impl Responder {
    info!(target: "control", "Received /stop request. Exiting...");

    tokio::spawn(async {
        tokio::time::sleep(Duration::from_secs(1)).await;
        std::process::exit(0);
    });

    HttpResponse::Ok().json(serde_json::json!({ "success": true, "data": null }))
}



#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let start = Instant::now();
    let start_data = web::Data::new(start);
    let config = load_config();

    init_tracing(&config.logs_dir, &config.name_for_port_manager);

    let port: u16 = config.static_port.expect("Port not defined in config for port manager");
    let ip = config.ip.clone();

    info!(target: "main", "Starting {} on {}:{}", &config.name_for_port_manager, ip, port);

    HttpServer::new(move || {
        App::new()
            .app_data(start_data.clone())
            .service(get_port_handler)
            .service(status_handler)
            .service(stop_handler)
    })
    .workers(4)
    .bind((ip.as_str(), port))?
    .run()
    .await
}
