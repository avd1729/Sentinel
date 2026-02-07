$job = Start-Job -ScriptBlock {
    cd 'd:\Rust Projects\sentinel'
    ./target/debug/sentinel.exe
}

Start-Sleep -Seconds 2

try {
    $response = Invoke-WebRequest http://localhost:8080 -UseBasicParsing -TimeoutSec 5
    Write-Host "Status: $($response.StatusCode)"
    Write-Host "Content:`n$($response.Content)"
} catch {
    Write-Host "Error: $_"
}

Stop-Job -Job $job
Remove-Job -Job $job
