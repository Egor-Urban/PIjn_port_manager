use std::collections::HashMap;
use std::fs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServiceInfo {
    pub ip: Option<String>,
    pub port: u16,
}


pub fn resolve_service_info(service_name: &str) -> Result<ServiceInfo, String> {
    let content = fs::read_to_string("ports.json")
        .map_err(|e| format!("Can't read ports.json: {}", e))?;

    let ports: HashMap<String, ServiceInfo> = serde_json::from_str(&content)
        .map_err(|e| format!("Error with JSON parsing: {}", e))?;

    ports.get(service_name)
        .cloned()
        .ok_or_else(|| format!("Service '{}' not found in ports.json", service_name))
}


pub fn update_service_ip(service_name: &str, new_ip: &str) -> Result<(), String> {
    let content = fs::read_to_string("ports.json")
        .map_err(|e| format!("Can't read ports.json: {}", e))?;

    let mut ports: HashMap<String, ServiceInfo> = serde_json::from_str(&content)
        .map_err(|e| format!("Error with JSON parsing: {}", e))?;

    let service = ports.get_mut(service_name)
        .ok_or_else(|| format!("Service '{}' not found in ports.json", service_name))?;

    service.ip = Some(new_ip.to_string());

    let new_content = serde_json::to_string_pretty(&ports)
        .map_err(|e| format!("Error serializing JSON: {}", e))?;

    fs::write("ports.json", new_content)
        .map_err(|e| format!("Can't write ports.json: {}", e))?;

    Ok(())
}
