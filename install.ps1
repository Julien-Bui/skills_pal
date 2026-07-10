$ErrorActionPreference = "Stop"

Write-Host "🚀 Téléchargement de Skills Pal pour Windows..." -ForegroundColor Cyan

$repo = "Julien-Bui/skills_pal"
$latestRelease = (Invoke-RestMethod -Uri "https://api.github.com/repos/$repo/releases/latest").tag_name

if (-not $latestRelease) {
    Write-Host "Impossible de trouver la dernière release." -ForegroundColor Red
    exit 1
}

$fileName = "skills_pal-x86_64-pc-windows-msvc.zip"
$downloadUrl = "https://github.com/$repo/releases/download/$latestRelease/$fileName"

$tempZip = "$env:TEMP\skills_pal.zip"
Invoke-WebRequest -Uri $downloadUrl -OutFile $tempZip

Write-Host "Extraction..."
$installDir = "$env:USERPROFILE\.skills_pal\bin"
if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Force -Path $installDir | Out-Null
}

Expand-Archive -Path $tempZip -DestinationPath $installDir -Force
Remove-Item $tempZip

# Ajouter au PATH utilisateur si ce n'est pas déjà fait
$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($userPath -notlike "*$installDir*") {
    $newPath = "$installDir;$userPath"
    [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
    Write-Host "⚠️ Le dossier a été ajouté à votre PATH. Vous devrez peut-être redémarrer votre terminal pour utiliser la commande." -ForegroundColor Yellow
}

Write-Host "✅ Skills Pal a été installé avec succès !" -ForegroundColor Green
Write-Host "👉 Lancez la commande 'skills_pal init' pour commencer." -ForegroundColor Cyan
