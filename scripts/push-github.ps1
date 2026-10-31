param(
    [Parameter(Mandatory = $true)][string]$Url
)

$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

git branch -M main
git remote remove origin 2>$null
git remote add origin $Url
git push -u origin main

Write-Host "Selesai. Buka tab Actions di repo GitHub untuk melihat CI."
