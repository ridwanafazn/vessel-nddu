# Build dan uji image HF secara lokal di port 7860.
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

docker build -f deploy/hf/Dockerfile -t vessel-hf:latest .
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

docker rm -f vessel-hf-test 2>$null
docker run -d --name vessel-hf-test -p 7860:7860 vessel-hf:latest
Start-Sleep -Seconds 4

Write-Host ""
Write-Host "Health:"
curl.exe -s http://127.0.0.1:7860/health
Write-Host ""
Write-Host "Swagger:"
curl.exe -s -o NUL -w "%{http_code}`n" http://127.0.0.1:7860/swagger-ui/
Write-Host ""
Write-Host "Stop dengan: docker rm -f vessel-hf-test"
