# Port Manager Microservice

**Developer:** Urban Egor  
**Version:** 4.4.14 r  

## Overview

This microservice provides an HTTP API to resolve service names into TCP port numbers, using a predefined mapping in `ports.json`. It is part of the PIjN protocol project.

## Features

- HTTP endpoint to retrieve the port for a given service.
- Status endpoint to verify the module is running.
- Logging of all access and errors with timestamps.
- Concurrent-safe file logging.

## Endpoints

### `GET /getport/{service_name}`

Resolves the provided `service_name` into its corresponding port.

#### Path Parameters
- `service_name`: Name of the service to resolve.

#### Response
- **200 OK** with JSON `{ "success": true, "data": <port_number> }` on success.
- **404 Not Found** with JSON `{ "success": false, "error": <error_message> }` on failure.

### `GET /status`

Returns a simple status check indicating the module is operational.

#### Response
- **200 OK** with JSON `{ "success": true, "data": null }`

## Configuration

- Runs on IP: `127.0.0.1`
- Port: `1030`
- Logs stored in `./logs/random_module_microservice_<DD_MM_YYYY>.log`
- Port mappings are read from `ports.json` in the root directory.

## Logging

All requests and internal messages are logged both to the console and to a file located in the `./logs/` directory. Logs include timestamp, source, and severity level.

## Internal Modules

### `port_manager`

Contains logic for reading and resolving port mappings from `ports.json`.

- `resolve_service_port(service_name: &str) -> Result<u16, String>`: Resolves the given service name to its port number.

## Dependencies

- `actix-web` - HTTP server and routing
- `chrono` - Timestamp formatting
- `once_cell` - Singleton static initialization
- `serde`, `serde_json` - JSON parsing
- `std::fs` - File I/O

## Error Handling

- If `ports.json` is missing or invalid, appropriate error messages are logged and returned.
- If a service name does not exist in `ports.json`, an error is returned to the client.
