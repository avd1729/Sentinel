# Sentinel

A high-performance HTTP web server and reverse proxy written in Rust with async I/O.

## Features

- ⚡ **Fast & Efficient** - Built on Tokio for high-performance async networking
- 🔧 **Configurable** - YAML-based configuration with hot-reload support
- 📁 **Static File Serving** - Serve static websites with custom error pages
- 🔄 **HTTP/1.1** - Full request/response handling with keep-alive support
- 📊 **Structured Logging** - Detailed tracing for debugging and monitoring

## Current Status

Sentinel is currently a **fully functional web server** capable of serving static files with HTTP/1.1 support. Future phases will add reverse proxy capabilities, load balancing, and advanced features.

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
- [ ] **Phase 2: Reverse Proxy** - Backend forwarding, routing, upstream connections
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
