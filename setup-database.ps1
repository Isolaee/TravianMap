# Travian Map Database Setup Script

Write-Host "🗄️  Setting up PostgreSQL Database for Travian Map" -ForegroundColor Green
Write-Host ""

# Check if Docker is installed
if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Docker is not installed or not in PATH" -ForegroundColor Red
    Write-Host "Please install Docker Desktop: https://docs.docker.com/desktop/install/windows-install/" -ForegroundColor Yellow
    exit 1
}

# Check if Docker is running
try {
    docker version | Out-Null
} catch {
    Write-Host "❌ Docker is not running. Please start Docker Desktop." -ForegroundColor Red
    exit 1
}

Write-Host "✅ Docker is available and running" -ForegroundColor Green

# Start PostgreSQL and Adminer using Docker Compose
Write-Host "🚀 Starting PostgreSQL database..." -ForegroundColor Yellow
docker-compose up -d

# Wait for database to be ready
Write-Host "⏳ Waiting for database to be ready..." -ForegroundColor Yellow
Start-Sleep -Seconds 10

# Check if containers are running
$postgres_status = docker-compose ps --services --filter "status=running" | Select-String "postgres"
$adminer_status = docker-compose ps --services --filter "status=running" | Select-String "adminer"

if ($postgres_status) {
    Write-Host "✅ PostgreSQL is running on port 5432" -ForegroundColor Green
    Write-Host "   Database: travian_map" -ForegroundColor Cyan
    Write-Host "   Username: postgres" -ForegroundColor Cyan
    Write-Host "   Password: password" -ForegroundColor Cyan
} else {
    Write-Host "❌ PostgreSQL failed to start" -ForegroundColor Red
}

if ($adminer_status) {
    Write-Host "✅ Adminer (Database Admin) is running on http://localhost:8080" -ForegroundColor Green
} else {
    Write-Host "❌ Adminer failed to start" -ForegroundColor Red
}

Write-Host ""
Write-Host "🔗 Connection Details:" -ForegroundColor Magenta
Write-Host "   Database URL: postgresql://postgres:password@localhost:5432/travian_map" -ForegroundColor White
Write-Host "   Adminer URL: http://localhost:8080" -ForegroundColor White
Write-Host ""
Write-Host "💡 Next Steps:" -ForegroundColor Magenta
Write-Host "   1. Your database is now ready!" -ForegroundColor White
Write-Host "   2. Start your Rust server: cd server && cargo run" -ForegroundColor White
Write-Host "   3. The server will automatically create tables and insert sample data" -ForegroundColor White
Write-Host "   4. Visit Adminer to view/manage your database" -ForegroundColor White
Write-Host ""
Write-Host "🛑 To stop the database:" -ForegroundColor Yellow
Write-Host "   docker-compose down" -ForegroundColor White
