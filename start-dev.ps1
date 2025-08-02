# Travian Map Development Starter
# This script starts both the Rust server and React client

Write-Host "🚀 Starting Travian Map Development Environment" -ForegroundColor Green
Write-Host ""

# Start Rust server in background
Write-Host "📦 Starting Rust server..." -ForegroundColor Yellow
Start-Job -Name "RustServer" -ScriptBlock {
    Set-Location "c:\Users\eero\Documents\Rust\TravianMap\server"
    cargo run
}

# Wait a moment for server to start
Start-Sleep -Seconds 2

# Start React client in background
Write-Host "⚛️  Starting React client..." -ForegroundColor Cyan
Start-Job -Name "ReactClient" -ScriptBlock {
    Set-Location "c:\Users\eero\Documents\Rust\TravianMap\client"
    npm run dev
}

Write-Host ""
Write-Host "✅ Services started!" -ForegroundColor Green
Write-Host "📡 Rust Server: http://127.0.0.1:3001" -ForegroundColor Yellow
Write-Host "🌐 React Client: http://127.0.0.1:5173" -ForegroundColor Cyan
Write-Host ""
Write-Host "📊 Job Status:" -ForegroundColor White
Get-Job

Write-Host ""
Write-Host "💡 Tips:" -ForegroundColor Magenta
Write-Host "  - Use 'Get-Job' to check job status"
Write-Host "  - Use 'Stop-Job RustServer' to stop the server"
Write-Host "  - Use 'Stop-Job ReactClient' to stop the client"
Write-Host "  - Use 'Remove-Job *' to clean up finished jobs"
