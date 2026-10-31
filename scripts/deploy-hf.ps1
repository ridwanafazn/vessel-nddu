param(
    [Parameter(Mandatory = $true)][string]$User,
    [Parameter(Mandatory = $true)][string]$Space,
    [string]$Token = $env:HF_TOKEN
)

if (-not $Token) {
    throw "Set environment variable HF_TOKEN lebih dulu, atau jalankan dengan -Token <token>."
}

$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

Write-Host "Menyiapkan branch hf..."
git checkout -B hf

Copy-Item "deploy\hf\Dockerfile" ".\Dockerfile" -Force
Copy-Item "deploy\hf\README.md" ".\README.md" -Force

git add Dockerfile README.md
git commit -m "deploy: hugging face spaces" --allow-empty

Write-Host "Push ke Hugging Face Space $User/$Space..."
git remote remove hf 2>$null
git remote add hf "https://$User`:$Token@huggingface.co/spaces/$User/$Space"
git push -f hf hf:main

git checkout master

Write-Host ""
Write-Host "Selesai. Buka:"
Write-Host "  https://$User-$Space.hf.space/swagger-ui/"
Write-Host "  https://$User-$Space.hf.space/api-docs/openapi.json"
Write-Host "  wss://$User-$Space.hf.space/ws"
