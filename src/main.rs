use actix_web::{get, web, App, HttpResponse, HttpServer, Responder, middleware::Logger, HttpRequest};
use chrono::Local;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::sync::Mutex;

mod port_manager;



struct LoggerService {
    file: Mutex<std::fs::File>,
}


impl LoggerService {
    fn new() -> Self {
        let date = Local::now().format("%d_%m_%Y");
        fs::create_dir_all("./logs").ok();
        let file_path = format!("./logs/random_module_microservice_{}.log", date);
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(file_path)
            .expect("Failed to open log file");

        Self { file: Mutex::new(file) }
    }


    fn log(&self, source: &str, level: &str, message: &str) {
        let now = Local::now().format("%d.%m.%Y %H:%M:%S");
        let entry = format!("[{}][{}][{}] {}\n", now, source, level, message);
        print!("{}", entry);
        if let Ok(mut file) = self.file.lock() {
            let _ = file.write_all(entry.as_bytes());
        }
    }
}


static LOGGER: Lazy<LoggerService> = Lazy::new(LoggerService::new);



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

    match port_manager::resolve_service_port(service_name) {
        Ok(port) => {
            LOGGER.log("get_port_handler", "INFO", &format!("Client {} get port {} for '{}'", client_addr, port, service_name));
            HttpResponse::Ok().json(serde_json::json!({ "success": true, "data": port }))
        }
        Err(err_msg) => {
            LOGGER.log("get_port_handler", "WARN", &format!("IP {} — {}", client_addr, err_msg));
            HttpResponse::NotFound().json(serde_json::json!({ "success": false, "error": err_msg }))
        }
    }
}



#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = 1030;
    let ip = "127.0.0.1";

    LOGGER.log("main", "INFO", &format!("Port manager microservice started on {}:{}", ip, port));

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(get_port_handler)
    })
    .workers(4)
    .bind((ip, port))?
    .run()
    .await
}
