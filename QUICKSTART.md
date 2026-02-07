# Quick Start Guide - Phase 2 Reverse Proxy

## Prerequisites

- Rust 1.70+ installed
- Python 3.x (for test backend servers)
- PowerShell (on Windows) or Bash (on Linux/Mac)

## Step 1: Build Sentinel

```powershell
cd "d:\Rust Projects\sentinel"
cargo build --release
```

The compiled binary will be at: `target/release/sentinel.exe`

## Step 2: Configure Proxy Mode

Edit `config.yaml` and uncomment the proxy section:

```yaml
server:
  listen_addr: "127.0.0.1:8080"

static_files:
  root: "public"
  index: "index.html"

# Reverse Proxy Configuration
proxy:
  backends:
    - url: "http://localhost:3000"
      name: "backend-1"
    - url: "http://localhost:3001"
      name: "backend-2"
    - url: "http://localhost:3002"
      name: "backend-3"
  
  connection_timeout_ms: 5000
  request_timeout_ms: 30000
```

## Step 3: Start Backend Servers

### Option A: Use the test script (Recommended)

```powershell
.\test_proxy.ps1
```

This will start 3 backend servers on ports 3000, 3001, and 3002.

### Option B: Start manually

In separate terminals:

```powershell
# Terminal 1
python backend_server.py 3000 backend-1

# Terminal 2
python backend_server.py 3001 backend-2

# Terminal 3
python backend_server.py 3002 backend-3
```

## Step 4: Run Sentinel

In a new terminal:

```powershell
cargo run --release
```

Or use the compiled binary:

```powershell
.\target\release\sentinel.exe
```

Expected output:
```
INFO Loaded configuration from config.yaml
INFO Initialized backend pool backends=3
INFO Listening on 127.0.0.1:8080
```

## Step 5: Test the Proxy

### Single Request

```powershell
curl http://localhost:8080/api/test
```

Expected response:
```json
{
  "backend": "backend-1",
  "port": 3000,
  "path": "/api/test",
  "method": "GET",
  "timestamp": 1234567890.123
}
```

### Load Balance Test

```powershell
.\test_load_balance.ps1
```

Expected output:
```
Testing Sentinel Proxy - Round Robin Distribution
=================================================

Sending 12 requests to http://localhost:8080...

Request 1 : Served by backend-1
Request 2 : Served by backend-2
Request 3 : Served by backend-3
Request 4 : Served by backend-1
Request 5 : Served by backend-2
Request 6 : Served by backend-3
...

Results:
--------
backend-1 : 4 requests (33.3%)
backend-2 : 4 requests (33.3%)
backend-3 : 4 requests (33.3%)
```

## Testing Failure Scenarios

### Test Backend Failure

1. Stop one backend (Ctrl+C in its terminal)
2. Make requests:
   ```powershell
   curl http://localhost:8080/api/test
   ```
3. Traffic will automatically route to remaining backends

### Test All Backends Down

1. Stop all backend servers
2. Make a request:
   ```powershell
   curl http://localhost:8080/api/test
   ```
3. Expected response: `503 Service Unavailable`

### Test Backend Recovery

1. Restart a stopped backend
2. Make requests - backend will automatically rejoin the pool

## Viewing Logs

Sentinel uses structured logging. Set log level:

```powershell
$env:RUST_LOG="debug"
cargo run
```

Log levels:
- `error` - Errors only
- `warn` - Warnings and errors
- `info` - General information (default)
- `debug` - Detailed debugging
- `trace` - Very verbose

## Switching Between Static and Proxy Mode

### Static File Mode
Comment out the `proxy` section in `config.yaml`:

```yaml
# proxy:
#   backends:
#     ...
```

Restart Sentinel - it will serve static files from `public/`

### Proxy Mode
Uncomment the `proxy` section and restart Sentinel

## Troubleshooting

### "No available backends"
- **Cause:** All backends are down
- **Solution:** Start at least one backend server

### "Connection timeout"
- **Cause:** Backend not responding
- **Solution:** Check backend is running, increase timeout in config

### "Address already in use"
- **Cause:** Port 8080 is already in use
- **Solution:** Change `listen_addr` in config or stop other process

### Backend not receiving traffic
- **Cause:** Backend marked as down after failures
- **Solution:** Restart backend - it will auto-recover on next successful request

## Performance Tips

1. **Connection Timeout:** Reduce for faster failure detection
   ```yaml
   connection_timeout_ms: 2000  # 2 seconds
   ```

2. **Request Timeout:** Increase for slow backends
   ```yaml
   request_timeout_ms: 60000  # 60 seconds
   ```

3. **Production Deployment:**
   - Use release build (`--release`)
   - Set appropriate log level (`RUST_LOG=info`)
   - Monitor backend health
   - Use multiple backends for redundancy

## Next Steps

- **Phase 3:** Load balancing with metrics and least connections
- **Phase 4:** Adaptive load balancing based on performance
- **Phase 5:** Rate limiting and caching
- **Phase 6:** TLS/HTTPS support

## Support

For issues or questions:
1. Check [docs/PHASE_2_PROXY.md](docs/PHASE_2_PROXY.md) for detailed documentation
2. Review test scripts for examples
3. Check logs for error messages
