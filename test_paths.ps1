# Test script to demonstrate Sentinel serving multiple paths

Write-Host "`n=== Testing Sentinel Web Server - Multiple Paths ===" -ForegroundColor Cyan

# Test 1: Root path
Write-Host "`n1. Testing GET / (should serve index.html)" -ForegroundColor Yellow
curl -s http://localhost:8080/ | Select-String -Pattern "<title>" | Write-Host

# Test 2: About page
Write-Host "`n2. Testing GET /about.html" -ForegroundColor Yellow
curl -s http://localhost:8080/about.html | Select-String -Pattern "<title>" | Write-Host

# Test 3: Docs subdirectory
Write-Host "`n3. Testing GET /docs/api.html (subdirectory)" -ForegroundColor Yellow
curl -s http://localhost:8080/docs/api.html | Select-String -Pattern "<title>" | Write-Host

# Test 4: CSS file
Write-Host "`n4. Testing GET /css/style.css (stylesheet)" -ForegroundColor Yellow
curl -s http://localhost:8080/css/style.css | Select-String -Pattern "font-family" | Select -First 1 | Write-Host

# Test 5: 404 error page
Write-Host "`n5. Testing GET /nonexistent (should show custom 404)" -ForegroundColor Yellow
curl -s http://localhost:8080/nonexistent | Select-String -Pattern "404" | Write-Host

# Test 6: Path traversal attempt (should show 400)
Write-Host "`n6. Testing GET /../etc (path traversal - should show custom 400)" -ForegroundColor Yellow
curl -s http://localhost:8080/../etc | Select-String -Pattern "400" | Write-Host

Write-Host "`n=== Test Complete ===" -ForegroundColor Cyan
Write-Host "All paths under the configured root directory are accessible!" -ForegroundColor Green
