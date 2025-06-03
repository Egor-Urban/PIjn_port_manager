use std::collections::HashMap;
use std::fs;



pub fn resolve_service_port(service_name: &str) -> Result<u16, String> {
    let content = fs::read_to_string("ports.json")
        .map_err(|e| format!("Cant read ports.json: {}", e))?;

    let ports: HashMap<String, u16> = serde_json::from_str(&content)
        .map_err(|e| format!("Error with JSON parsing: {}", e))?;

    ports.get(service_name)
        .copied()
        .ok_or_else(|| format!("Service '{}' not found in ports.json", service_name))
}
