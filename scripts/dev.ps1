# Jalankan dev server (foreground) di 0.0.0.0 agar bisa diakses dari LAN.
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
$env:HTTP_ADDR = "0.0.0.0:8080"
$env:WS_ADDR = "0.0.0.0:8081"
$env:TCP_ADDR = "0.0.0.0:9001"
$env:RUST_LOG = "info"

Write-Host "REST    : http://127.0.0.1:8080"
Write-Host "Swagger : http://127.0.0.1:8080/swagger-ui/"
Write-Host "WS      : ws://127.0.0.1:8081"
Write-Host ""

cargo run
