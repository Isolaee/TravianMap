# Travian Map - Empty Database Setup Script
# This script sets up a clean, empty PostgreSQL database for Travian Map

Write-Host "🗄️  Setting up Clean PostgreSQL Database for Travian Map" -ForegroundColor Green
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

# Clean up any existing containers and volumes
Write-Host "🧹 Cleaning up existing containers and data..." -ForegroundColor Yellow
docker-compose down -v 2>$null
docker volume rm travianmap_postgres_data 2>$null

# Start PostgreSQL with empty schema
Write-Host "🚀 Starting PostgreSQL database with empty schema..." -ForegroundColor Yellow
docker-compose up -d

# Wait for database to be ready
Write-Host "⏳ Waiting for database to be ready..." -ForegroundColor Yellow
Start-Sleep -Seconds 15

# Check if containers are running
$postgres_status = docker-compose ps --services --filter "status=running" | Select-String "postgres"
$adminer_status = docker-compose ps --services --filter "status=running" | Select-String "adminer"

if ($postgres_status) {
    Write-Host "✅ PostgreSQL is running on port 5432" -ForegroundColor Green
    Write-Host "   Database: travian_map (EMPTY - No sample data)" -ForegroundColor Cyan
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
Write-Host "📝 Schema Management:" -ForegroundColor Magenta
Write-Host "   Use schema.sql to manually create/modify the database schema" -ForegroundColor White
Write-Host "   Run: psql -h localhost -U postgres -d travian_map -f schema.sql" -ForegroundColor White
Write-Host ""
Write-Host "💡 Next Steps:" -ForegroundColor Magenta
Write-Host "   1. Your EMPTY database is now ready!" -ForegroundColor White
Write-Host "   2. Add your Travian servers through the API or directly in database" -ForegroundColor White
Write-Host "   3. Start your Rust server: cd server && cargo run" -ForegroundColor White
Write-Host "   4. The server will load real data from your configured Travian servers" -ForegroundColor White
Write-Host "   5. Use Adminer to view/manage your database" -ForegroundColor White
Write-Host ""
Write-Host "🛑 To stop the database:" -ForegroundColor Yellow
Write-Host "   docker-compose down" -ForegroundColor White
Write-Host ""
Write-Host "🎯 Database is EMPTY and ready for real Travian data!" -ForegroundColor Green
