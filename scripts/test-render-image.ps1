param(
    [int]$Port = 10000
)

$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

docker build -f deploy/hf/Dockerfile -t vessel-render:latest .
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

docker rm -f vessel-render-test 2>$null
docker run -d --name vessel-render-test --user 1000:1000 `
    -e PORT=$Port -p "${Port}:${Port}" vessel-render:latest

Start-Sleep -Seconds 5

Write-Host ""
Write-Host "Health:"
curl.exe -s "http://127.0.0.1:$Port/health"
Write-Host ""
Write-Host "Swagger:"
curl.exe -s -o NUL -w "%{http_code}`n" "http://127.0.0.1:$Port/swagger-ui/"
Write-Host ""
Write-Host "WebSocket: ws://127.0.0.1:$Port/ws"
Write-Host "Stop dengan: docker rm -f vessel-render-test"
