$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

$pidFile = "target\debug\vessel-dev.pid"
if (Test-Path $pidFile) {
    $serverPid = Get-Content $pidFile -ErrorAction SilentlyContinue
    if ($serverPid) {
        Stop-Process -Id $serverPid -Force -ErrorAction SilentlyContinue
        Write-Host "stopped pid=$serverPid"
    }
    Remove-Item $pidFile -Force -ErrorAction SilentlyContinue
} else {
    Get-Process vessel-dev -ErrorAction SilentlyContinue | Stop-Process -Force
    Write-Host "stopped vessel-dev"
}
