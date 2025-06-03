# [PIjN] Port Manager Microservice

Developer: Urban Egor  
Version: 6.5.21 r  

## Overview

This microservice provides an HTTP API to resolve service names into TCP port numbers,
using a predefined mapping in `ports.json`. It is part of the PIjN protocol project.

## Features

- HTTP endpoint to retrieve the port for a given service.
- Status endpoint to verify the module is running.
- Graceful shutdown via `/stop` endpoint.
- Structured logging with daily log files.
- System status monitoring (CPU, RAM, Disk, Uptime).

## Endpoints

### GET /getport/{service_name}

Resolves the provided `service_name` into its corresponding port.

Path Parameters:
- service_name: Name of the service to resolve.

Response:
- 200 OK: { "success": true, "data": <port_number> }
- 404 Not Found: { "success": false, "error": <error_message> }

### GET /status

Returns the current system status and uptime.

Response:
- 200 OK:
  {
    "success": true,
    "data": {
      "uptime": <seconds>,
      "cpu": <percent>,
      "ram": <percent>,
      "disk": <percent>
    }
  }

### GET /stop

Triggers a graceful shutdown of the service after a short delay.

Response:
- 200 OK: { "success": true, "data": null }

## Configuration

- IP address: 127.0.0.1
- Port: Defined via `static_port` in `config.json` (e.g., 1030)
- Logs stored in: ./logs/<name_for_port_manager>_<DD_MM_YYYY>.log
- Port mappings source: ports.json (must be present in the root directory)

Example config.json:
{
  "ip": "127.0.0.1",
  "static_port": 1030,
  "name_for_port_manager": "port_manager",
  "logs_dir": "./logs"
}

## Logging

All requests and internal events are logged with timestamps and severity levels.
Logs are written to rotating daily files in the specified `logs_dir` path.

## Internal Modules

- port_manager: Contains logic for reading and resolving service-to-port mappings.
  - fn resolve_service_port(service_name: &str) -> Result<u16, String>

- utils: Handles configuration loading and log initialization.

- status: Collects and returns system health metrics (uptime, CPU, RAM, disk usage).

## Dependencies

- actix-web: Web framework
- chrono: Date/time formatting
- reqwest: HTTP client (optional)
- serde, serde_json: Configuration and response serialization
- sysinfo: System monitoring
- tracing: Structured logging

## Error Handling

- Missing or malformed ports.json results in 500-level errors with meaningful messages.
- Nonexistent service names return 404 errors.
- All errors are logged with detailed context.
