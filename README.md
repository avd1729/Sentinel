# Sentinel - Adaptive Reverse Proxy

A high-performance, multi-phase HTTP reverse proxy server written in Rust using Tokio for async I/O.

## Project Overview

Sentinel is being developed in phases to progressively build a feature-rich reverse proxy:

- **Phase 0** (Foundation): Core project setup, TCP listener, logging
- **Phase 1** (HTTP Core): HTTP/1.1 parser, request/response handling, keep-alive support
- **Phase 2** (Reverse Proxy): Backend proxying, routing, upstream connections
- **Phase 3** (Load Balancing): Load balancing algorithms, metrics collection
- **Phase 4+** (Advanced): Health checks, caching, SSL/TLS, etc.

Currently on: **Phase 1 - HTTP Core**

## Quick Start

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
- Cargo (comes with Rust)

### Building

```bash
# Clone the repository
git clone <repository-url>
cd sentinel

# Build the project
cargo build --release

# Run tests
cargo test
```

### Running the Server

```bash
# Start with default configuration (127.0.0.1:8080)
./target/release/sentinel

# Start with custom address via environment variable
LISTEN=0.0.0.0:3000 ./target/release/sentinel

# Use a YAML configuration file
# Create config.yaml (or copy from config.example.yaml) and run:
./target/release/sentinel
```

The server will start listening and log its status:
```
2026-01-31T16:25:01.632591Z  INFO Loaded configuration from config.yaml
2026-01-31T16:25:01.632591Z  INFO Listening on 127.0.0.1:8080
```

### Testing the Server

```powershell
# Using curl
curl http://localhost:8080

# Using PowerShell
Invoke-WebRequest http://localhost:8080

# Using Python
python -c "
import socket
s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
s.connect(('localhost', 8080))
s.sendall(b'GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n')
response = s.recv(4096)
print(response.decode())
s.close()
"
```

## Project Structure

```
sentinel/
├── src/
│   ├── lib.rs              # Library exports
│   ├── main.rs             # Application entry point
│   ├── config.rs           # Configuration loading
│   ├── http/               # HTTP protocol module
│   │   ├── mod.rs          # HTTP module root
│   │   ├── connection.rs   # Connection state machine
│   │   ├── parser.rs       # HTTP request parser
│   │   ├── request.rs      # HTTP request types
│   │   ├── response.rs     # HTTP response types
│   │   ├── writer.rs       # HTTP response writer
│   │   └── mime.rs         # MIME type detection
│   └── server/
│       ├── mod.rs          # Server module root
│       └── listener.rs     # TCP listener
├── tests/                  # Integration tests
│   ├── test_config.rs
│   ├── test_parser.rs
│   ├── test_request.rs
│   └── test_response.rs
├── public/                 # Static files for serving
│   └── index.html
├── config.yaml            # Server configuration
├── config.example.yaml    # Example configuration
├── Cargo.toml             # Project manifest
└── README.md              # This file
```

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --test test_parser
cargo test --test test_request
cargo test --test test_response
cargo test --test test_config

# Run with output
cargo test -- --nocapture

# Run a single test
cargo test test_parse_simple_get
```

### Building Documentation

```bash
# Generate and open documentation
cargo doc --open
```

### Code Formatting

```bash
# Check formatting
cargo fmt --check

# Auto-format code
cargo fmt
```

### Linting

```bash
# Check for common mistakes
cargo clippy
```

## Current Features (Phase 1)

### HTTP Protocol Support

- ✅ HTTP/1.1 request parsing
- ✅ Request methods: GET, POST, PUT, DELETE, HEAD, OPTIONS, PATCH
- ✅ Header parsing and handling
- ✅ Request body support with Content-Length
- ✅ HTTP response generation with status codes
- ✅ Keep-Alive connection support (HTTP/1.1 default)
- ✅ Connection multiplexing on same socket

### Server Capabilities

- ✅ Async TCP listener (Tokio-based)
- ✅ YAML-based configuration system
- ✅ Static file serving from configurable directory
- ✅ Configurable default index file
- ✅ Automatic MIME type detection
- ✅ Structured logging with tracing
- ✅ Configuration via YAML file or environment variables
- ✅ Graceful shutdown on Ctrl+C

### Testing

- ✅ 41+ unit and integration tests
- ✅ Config parsing tests
- ✅ HTTP parser tests
- ✅ Request handling tests
- ✅ Response building tests

## Configuration

Sentinel supports multiple configuration methods, with the following priority:

1. **YAML Configuration File** (highest priority)
2. **Environment Variables**
3. **Default Values** (lowest priority)

### YAML Configuration

Create a `config.yaml` file in the project root:

```yaml
# Server listening configuration
server:
  listen_addr: "127.0.0.1:8080"

# Static file serving configuration
static_files:
  # Root directory for serving static files
  root: "public"
  
  # Default file to serve when a directory is requested
  index: "index.html"
  
  # Enable directory listing (not yet implemented)
  directory_listing: false
```

See `config.example.yaml` for more configuration options.

### Environment Variables

If no `config.yaml` is found, environment variables are used:

| Variable | Default | Description |
|----------|---------|-------------|
| `LISTEN` | `127.0.0.1:8080` | Server bind address and port |

Example:
```bash
LISTEN=0.0.0.0:8080 cargo run
```

### Default Configuration

If no config file or environment variables are set, Sentinel uses:
- Listen address: `127.0.0.1:8080`
- Static files root: `public`
- Default index: `index.html`

## Architecture

### Connection State Machine

Each client connection progresses through well-defined states:

```
Reading → Processing → Writing → [Closed or Reading for keep-alive]
```

1. **Reading**: Waits for and parses incoming HTTP requests
2. **Processing**: Handles the request and generates a response
3. **Writing**: Sends the HTTP response to the client
4. **Closed**: Connection terminates

For keep-alive connections, after Writing, the state returns to Reading for the next request.

### HTTP Request Flow

```
[Client] 
    ↓ (TCP Socket)
[TcpListener - Accepts connection]
    ↓
[Connection - State Machine]
    ↓
[Parser - Extracts method, path, headers]
    ↓
[Request Handler - Generates response]
    ↓
[ResponseWriter - Serializes and sends HTTP response]
    ↓
[Back to Client]
```

## Logging

Sentinel uses structured logging via the `tracing` crate. Logs include:

- Connection acceptance and closure
- HTTP requests received (method, path)
- Request duration and status codes
- Errors and warnings

Example log output:
```
2026-01-31T16:25:01.632591Z  INFO Listening on 127.0.0.1:8080
2026-01-31T16:25:10.407530Z  INFO method=GET path=/ Received HTTP request
2026-01-31T16:25:10.408530Z  INFO method=GET path=/ status=200 duration_ms=1 HTTP request completed
```

## Next Steps (Phase 2)

The next phase will add:

- Backend server configuration
- Request forwarding to upstream servers
- Response streaming from backends
- Basic routing logic
- Error handling for backend failures
- Request/response timing metrics

## Performance Considerations

- Async I/O with Tokio for handling thousands of concurrent connections
- Non-blocking request parsing with buffering
- Zero-copy where possible for request/response forwarding (Phase 2+)
- Connection pooling for backend servers (Phase 2+)

## Dependencies

Key dependencies:

- **tokio**: Async runtime
- **serde**: Serialization framework  
- **serde_yaml**: YAML configuration parsing
- **tracing**: Structured logging
- **anyhow**: Error handling

See `Cargo.toml` for full dependency list.


**Current Status**: Phase 1 - HTTP Core ✅  
**Last Updated**: January 31, 2026
