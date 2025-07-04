/*
Port manager microservice for PIjN protocol project
Developer: Urban Egor
Version: 6.5.21 r
*/



mod status;
mod utils;
mod port_manager;

use status::get_status;
use utils::{init_tracing, load_config};
use port_manager::{resolve_service_info, update_service_ip};

use actix_web::{dev::{ServiceRequest, ServiceResponse, Transform, Service}, get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder, Error, body::BoxBody};
use serde::{Deserialize, Serialize};
use futures::future::{ok, Ready, LocalBoxFuture};
use std::task::{Context, Poll};
use std::net::IpAddr;
use std::rc::Rc;
use std::time::Instant;
use tokio::time::Duration;
use tracing::{info, warn};



// --- local network protect ---


pub struct LocalNetworkOnly;

impl<S> Transform<S, ServiceRequest> for LocalNetworkOnly
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = LocalNetworkOnlyMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(LocalNetworkOnlyMiddleware {
            service: Rc::new(service),
        })
    }
}

pub struct LocalNetworkOnlyMiddleware<S> {
    service: Rc<S>,
}

impl<S> Service<ServiceRequest> for LocalNetworkOnlyMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = Rc::clone(&self.service);

        let ip_opt = req.connection_info().realip_remote_addr()
            .and_then(|addr| addr.split(':').next())
            .and_then(|ip_str| ip_str.parse::<IpAddr>().ok());

        let allowed = match ip_opt {
            Some(ip) => is_local_ip(&ip),
            None => false,
        };

        if allowed {
            Box::pin(async move { svc.call(req).await })
        } else {
            Box::pin(async move {
                let (req, _) = req.into_parts();

                let resp = HttpResponse::Forbidden()
                    .body("Access denied: only local network allowed");

                let srv_resp = ServiceResponse::new(req, resp);

                Ok(srv_resp)
            })
        }
    }
}

fn is_local_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => ipv4.is_loopback() || ipv4.is_private(),
        IpAddr::V6(ipv6) => ipv6.is_loopback(),
    }
}


// --- local network protect ---


#[derive(Serialize, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: T,
}


#[derive(Deserialize)]
struct PortRequest {
    service_name: String,
    ip: String,
}



#[post("/getport")]
async fn get_port_handler(req: HttpRequest, json: web::Json<PortRequest>) -> impl Responder {
    let client_addr = req.peer_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let service_name = &json.service_name;
    let client_ip_in_body = &json.ip;

    match resolve_service_info(service_name) {
        Ok(info) => {
            let port = info.port;

            // Обновим IP в файле
            match update_service_ip(service_name, client_ip_in_body) {
                Ok(_) => {
                    info!(target: "get_port_handler",
                          "Client (TCP: {}) reported IP {} — saved for '{}' with port {}",
                          client_addr, client_ip_in_body, service_name, port);

                    HttpResponse::Ok().json(serde_json::json!({ "success": true, "data": port }))
                }
                Err(err) => {
                    warn!(target: "get_port_handler",
                          "Failed to update IP for {} (TCP: {} reported IP {}): {}",
                          service_name, client_addr, client_ip_in_body, err);
                    HttpResponse::InternalServerError()
                        .json(serde_json::json!({ "success": false, "error": err }))
                }
            }
        }
        Err(err_msg) => {
            warn!(target: "get_port_handler",
                  "Client (TCP: {}) reported IP {} — {}",
                  client_addr, client_ip_in_body, err_msg);

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
            .wrap(LocalNetworkOnly)  
            .service(get_port_handler)
            .service(status_handler)
            .service(stop_handler)
    })
    .workers(config.workers_count)
    .bind((ip.as_str(), port))?
    .run()
    .await
}
