# Test proxy with multiple requests to verify round-robin

Write-Host "Testing Sentinel Proxy - Round Robin Distribution" -ForegroundColor Cyan
Write-Host "=================================================" -ForegroundColor Cyan
Write-Host ""

$proxyUrl = "http://localhost:8080"
$requests = 12

Write-Host "Sending $requests requests to $proxyUrl..." -ForegroundColor Yellow
Write-Host ""

$backendCounts = @{}

for ($i = 1; $i -le $requests; $i++) {
    try {
        $response = Invoke-WebRequest -Uri "$proxyUrl/api/test" -UseBasicParsing
        $json = $response.Content | ConvertFrom-Json
        
        $backend = $json.backend
        
        if ($backendCounts.ContainsKey($backend)) {
            $backendCounts[$backend]++
        } else {
            $backendCounts[$backend] = 1
        }
        
        Write-Host "Request $i : Served by $backend" -ForegroundColor Green
    } catch {
        Write-Host "Request $i : FAILED - $($_.Exception.Message)" -ForegroundColor Red
    }
    
    Start-Sleep -Milliseconds 100
}

Write-Host ""
Write-Host "Results:" -ForegroundColor Cyan
Write-Host "--------" -ForegroundColor Cyan

foreach ($backend in $backendCounts.Keys | Sort-Object) {
    $count = $backendCounts[$backend]
    $percentage = [math]::Round(($count / $requests) * 100, 1)
    Write-Host "$backend : $count requests ($percentage%)" -ForegroundColor White
}

Write-Host ""
Write-Host "Expected: Each backend should receive approximately 33% of requests" -ForegroundColor Yellow
