# Test Sentinel Reverse Proxy
# This script starts multiple backend servers and tests the proxy

Write-Host "Sentinel Reverse Proxy - Phase 2 Test Script" -ForegroundColor Cyan
Write-Host "=============================================" -ForegroundColor Cyan
Write-Host ""

# Check if Python is available
if (-not (Get-Command python -ErrorAction SilentlyContinue)) {
    Write-Host "ERROR: Python is not installed or not in PATH" -ForegroundColor Red
    exit 1
}

Write-Host "Starting backend servers..." -ForegroundColor Yellow

# Start backend servers in separate processes
$backend1 = Start-Process python -ArgumentList "backend_server.py", "3000", "backend-1" -PassThru -NoNewWindow
$backend2 = Start-Process python -ArgumentList "backend_server.py", "3001", "backend-2" -PassThru -NoNewWindow
$backend3 = Start-Process python -ArgumentList "backend_server.py", "3002", "backend-3" -PassThru -NoNewWindow

Write-Host "Backend servers started:" -ForegroundColor Green
Write-Host "  - backend-1 on port 3000" -ForegroundColor Green
Write-Host "  - backend-2 on port 3001" -ForegroundColor Green
Write-Host "  - backend-3 on port 3002" -ForegroundColor Green
Write-Host ""

# Wait for backends to start
Start-Sleep -Seconds 2

Write-Host "Testing backends..." -ForegroundColor Yellow

# Test each backend
try {
    $response1 = Invoke-WebRequest -Uri "http://localhost:3000/" -UseBasicParsing
    Write-Host "✓ backend-1 responding" -ForegroundColor Green
    
    $response2 = Invoke-WebRequest -Uri "http://localhost:3001/" -UseBasicParsing
    Write-Host "✓ backend-2 responding" -ForegroundColor Green
    
    $response3 = Invoke-WebRequest -Uri "http://localhost:3002/" -UseBasicParsing
    Write-Host "✓ backend-3 responding" -ForegroundColor Green
} catch {
    Write-Host "ERROR: Backends not responding properly" -ForegroundColor Red
    Write-Host "Stopping backend servers..." -ForegroundColor Yellow
    Stop-Process -Id $backend1.Id -Force
    Stop-Process -Id $backend2.Id -Force
    Stop-Process -Id $backend3.Id -Force
    exit 1
}

Write-Host ""
Write-Host "Backend servers are ready!" -ForegroundColor Green
Write-Host ""
Write-Host "Now you can:" -ForegroundColor Cyan
Write-Host "1. Update config.yaml to enable proxy mode (uncomment the proxy section)" -ForegroundColor White
Write-Host "2. Run 'cargo run' to start Sentinel" -ForegroundColor White
Write-Host "3. Test with: curl http://localhost:8080/" -ForegroundColor White
Write-Host ""
Write-Host "Press Ctrl+C to stop all backend servers..." -ForegroundColor Yellow

# Wait for Ctrl+C
try {
    while ($true) {
        Start-Sleep -Seconds 1
    }
} finally {
    Write-Host ""
    Write-Host "Stopping backend servers..." -ForegroundColor Yellow
    Stop-Process -Id $backend1.Id -Force -ErrorAction SilentlyContinue
    Stop-Process -Id $backend2.Id -Force -ErrorAction SilentlyContinue
    Stop-Process -Id $backend3.Id -Force -ErrorAction SilentlyContinue
    Write-Host "All backend servers stopped" -ForegroundColor Green
}
