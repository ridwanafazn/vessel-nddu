# Build lalu jalankan dev server di background.
# Binary disalin ke vessel-dev.exe agar `cargo build` berikutnya tidak terkunci.
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"

$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

if (Test-Path "target\debug\vessel-dev.pid") {
    $old = Get-Content "target\debug\vessel-dev.pid" -ErrorAction SilentlyContinue
    if ($old) { Stop-Process -Id $old -Force -ErrorAction SilentlyContinue }
}

cargo build
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Copy-Item "target\debug\vessel.exe" "target\debug\vessel-dev.exe" -Force

$env:HTTP_ADDR = "0.0.0.0:8080"
$env:WS_ADDR = "0.0.0.0:8081"
$env:TCP_ADDR = "0.0.0.0:9001"
$env:RUST_LOG = "info"

$p = Start-Process -FilePath "target\debug\vessel-dev.exe" `
    -RedirectStandardOutput "target\debug\vessel-dev.out.log" `
    -RedirectStandardError "target\debug\vessel-dev.err.log" `
    -PassThru -NoNewWindow

$p.Id | Set-Content "target\debug\vessel-dev.pid"

Write-Host "dev server pid=$($p.Id)"
Write-Host "REST    : http://127.0.0.1:8080"
Write-Host "Swagger : http://127.0.0.1:8080/swagger-ui/"
Write-Host "WS      : ws://127.0.0.1:8081"
