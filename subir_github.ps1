<#
.SYNOPSIS
Crea el repositorio en GitHub y sube todo el proyecto (código + documentación).

.DESCRIPTION
Requiere Git y GitHub CLI instalados. Si aún no iniciaste sesión, ejecuta primero:
    gh auth login

Ejemplos:
    .\subir_github.ps1
    .\subir_github.ps1 -RepoName servidor-https-rust -Visibility private
#>
param(
    [string]$RepoName = "servidor-https-rust",
    [ValidateSet("public", "private", "internal")]
    [string]$Visibility = "public",
    [string]$CommitMessage = "Servidor HTTPS multihilo en Rust con pool de hilos y rustls"
)

$ErrorActionPreference = "Stop"
Set-Location -Path $PSScriptRoot

# El instalador a veces no refresca el PATH de la sesión actual: añadimos
# las rutas típicas de Git y GitHub CLI en Windows.
foreach ($dir in @("C:\Program Files\Git\cmd", "C:\Program Files\GitHub CLI")) {
    if ((Test-Path $dir) -and ($env:PATH -notlike "*$dir*")) {
        $env:PATH = "$dir;$env:PATH"
    }
}

# 1. Comprobar herramientas
foreach ($tool in @("git", "gh")) {
    if (-not (Get-Command $tool -ErrorAction SilentlyContinue)) {
        throw "No encontré '$tool' en el PATH. Instálalo y vuelve a ejecutar el script."
    }
}

# 2. Comprobar sesión de GitHub CLI
$auth = gh auth status 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "No hay sesión activa en GitHub CLI. Ejecuta 'gh auth login' y vuelve a correr este script." -ForegroundColor Yellow
    gh auth login
}

$usuario = gh api user --jq .login
Write-Host "Autenticado como $usuario" -ForegroundColor Green

# 3. Inicializar repo local
if (-not (Test-Path .git)) {
    git init -b main
}
git add .
$hayCambios = git diff --cached --quiet
if ($LASTEXITCODE -ne 0) {
    git commit -m $CommitMessage
} else {
    Write-Host "No hay cambios nuevos para commitear." -ForegroundColor Yellow
}

# 4. Crear el repo remoto si no existe y hacer push
$existe = gh repo view "$usuario/$RepoName" 2>$null
if ($LASTEXITCODE -eq 0) {
    Write-Host "El repo $usuario/$RepoName ya existe; actualizando." -ForegroundColor Yellow
    $remote = git remote get-url origin 2>$null
    if ($LASTEXITCODE -ne 0) {
        git remote add origin "https://github.com/$usuario/$RepoName.git"
    }
} else {
    gh repo create $RepoName "--$Visibility" --source=. --remote=origin
}

git push -u origin main
Write-Host "Listo: https://github.com/$usuario/$RepoName" -ForegroundColor Green
