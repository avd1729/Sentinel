# Sentinel

A high-performance HTTP web server and adaptive reverse proxy written in Rust with async I/O.

## Features

### Current Features (Phase 1-2)

- ⚡ **Fast & Efficient** - Built on Tokio for high-performance async networking
- 🔧 **Configurable** - YAML-based configuration with hot-reload support
- 📁 **Static File Serving** - Serve static websites with custom error pages
- 🔄 **HTTP/1.1** - Full request/response handling with keep-alive support
- 🔀 **Reverse Proxy** - Forward requests to multiple backend servers
- ⚖️ **Load Balancing** - Round-robin distribution across backends
- 🛡️ **Fault Tolerance** - Automatic backend failure detection and recovery
- ⏱️ **Timeout Handling** - Configurable connection and request timeouts
- 📊 **Structured Logging** - Detailed tracing for debugging and monitoring

### Coming Soon

- 🎯 **Adaptive Load Balancing** - Performance-based backend selection (Phase 4)
- 💾 **Response Caching** - Cache frequently requested content (Phase 5)
- 🚦 **Rate Limiting** - Per-IP request throttling (Phase 5)
- 🔒 **TLS Termination** - HTTPS support with certificate management (Phase 6)

## Current Status

Sentinel is now a **fully functional reverse proxy** with:
- Multiple backend server support
- Automatic failover and recovery
- Round-robin load balancing
- Comprehensive error handling
- Production-ready static file serving

**Phase 1 (HTTP Server)**: ✅ Complete  
**Phase 2 (Reverse Proxy)**: ✅ Complete  
**Phase 3 (Load Balancing)**: 🚧 In Progress

## Quick Start

```bash
# Clone the repository
git clone https://github.com/yourusername/sentinel.git
cd sentinel

# Build
cargo build --release

# Run
./target/release/sentinel
```

Visit `http://localhost:8080` in your browser!

## Installation

### Prerequisites

- Rust 1.70 or higher
- Cargo (included with Rust)

### From Source

```bash
cargo install --path .
```

## Configuration

Create a `config.yaml` file:

### Static File Server Mode

```yaml
server:
  listen_addr: "127.0.0.1:8080"

static_files:
  root: "public"
  index: "index.html"
  error_pages:
    not_found: "errors/404.html"
    bad_request: "errors/400.html"
```

### Reverse Proxy Mode

```yaml
server:
  listen_addr: "127.0.0.1:8080"

### Running Tests

```bash
# Run all tests
cargo test

# Run backend tests
cargo test --lib proxy

# Format code
cargo fmt

# Lint
cargo clippy

# Generate docs
cargo doc --open
```

### Testing the Reverse Proxy

1. **Start backend servers:**

```powershell
# Using the test script (starts 3 backends)
.\test_proxy.ps1
```

Or manually:

```powershell
python backend_server.py 3000 backend-1
python backend_server.py 3001 backend-2
python backend_server.py 3002 backend-3
```

2. **Enable proxy in config.yaml** (uncomment the proxy section)

3. **Run Sentinel:**

```bash
cargo run
```

4. **Test requests:**

```powershell
# Single request
curl http://localhost:8080/

# Test load balancing
### Completed

- [x] **Phase 0: Foundation** - Project setup, TCP listener, basic logging
- [x] **Phase 1: HTTP Core** - HTTP parser, response writer, static file serving
- [x] **Phase 2: Reverse Proxy** - Backend forwarding, routing, error handling

### In Progress / Planned

- [ ] **Phase 3: Load Balancing Core** - Metrics, least connections algorithm
- [ ] **Phase 4: Adaptive Load Balancer** ⭐ - EWMA latency, performance-based routing
- [ ] **Phase 5: Rate Limiting + Caching** - Token bucket, LRU cache, TTL support
- [ ] **Phase 6: TLS Termination** - HTTPS support, certificate management
- [ ] **Phase 7: Advanced Features** - Circuit breakers, health checks, WASM plugins
- [ ] **Phase 8: Observability** - Prometheus metrics, structured logging, benchmarks
- [ ] **Phase 9: Hardening** - Security review, graceful shutdown, resource cleanup

## Project Structure

```
sentinel/
├── src/
│   ├── main.rs              # Application entry point
│   ├── config.rs            # Configuration management
│   ├── http/                # HTTP protocol implementation
│   │   ├── connection.rs    # Connection state machine
│   │   ├── parser.rs        # HTTP request parser
│   │   ├── response.rs      # HTTP response builder
│   │   └── writer.rs        # Response writer
│   ├── proxy/               # Reverse proxy implementation
│   │   ├── backend.rs       # Backend pool and state management
│   │   └── upstream.rs      # Request forwarding logic
│   └── server/              # Server implementation
│       └── listener.rs      # TCP listener and connection handling
├── public/                  # Static files directory
├── docs/                    # Documentation
│   └── PHASE_2_PROXY.md    # Phase 2 documentation
├── config.yaml              # Server configuration
└── Cargo.toml              # Rust dependencies
```

## Documentation

- [Phase 2: Reverse Proxy Documentation](docs/PHASE_2_PROXY.md)
- [Task Breakdown](specs/TASK_BREAKDOWN.md)
- [Product Requirements](specs/PRD.md)

- Requests are distributed evenly across backends (round-robin)
- Failed backends are automatically skipped
- Backends recover automatically on successful requests
- 502 Bad Gateway when all backends are down
- 504 Gateway Timeout on backend timeouts   name: "backend-3"
  
  connection_timeout_ms: 5000
  request_timeout_ms: 30000
```

### Configuration Options

| Section | Option | Description | Default |
|---------|--------|-------------|---------|
| `server` | `listen_addr` | Address to bind to | Required |
| `proxy` | `backends` | List of backend servers | Optional |
| `proxy` | `connection_timeout_ms` | Backend connection timeout | 5000 |
| `proxy` | `request_timeout_ms` | Backend request timeout | 30000 |

Or use environment variables:

```bash
LISTEN=0.0.0.0:3000 sentinel
```

## Development

```bash
# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy

# Generate docs
cargo doc --open
```

## Roadmap

- [x] **Phase 1: Web Server** - Static file serving, HTTP/1.1, keep-alive
- [x] **Phase 2: Reverse Proxy** - Backend forwarding, routing, upstream connections
- [ ] **Phase 3: Load Balancing** - Round-robin, least connections, weighted distribution
- [ ] **Phase 4: Advanced Features** - Health checks, caching, rate limiting, SSL/TLS

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Tokio](https://tokio.rs/) - Async runtime for Rust
- Inspired by Nginx and other modern web servers
