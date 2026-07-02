# ORBIT Screen Sharing Test Script
# This script helps you set up and test the complete ORBIT screen sharing flow

Write-Host "=== ORBIT Screen Sharing Test Script ===" -ForegroundColor Cyan
Write-Host ""

# Check prerequisites
Write-Host "Checking prerequisites..." -ForegroundColor Yellow

# Check PostgreSQL
try {
    $pgResult = & psql --version 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✓ PostgreSQL found: $pgResult" -ForegroundColor Green
    } else {
        Write-Host "✗ PostgreSQL not found. Please install PostgreSQL." -ForegroundColor Red
        exit 1
    }
} catch {
    Write-Host "✗ PostgreSQL not found. Please install PostgreSQL." -ForegroundColor Red
    exit 1
}

# Check Node.js
try {
    $nodeResult = & node --version 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✓ Node.js found: $nodeResult" -ForegroundColor Green
    } else {
        Write-Host "✗ Node.js not found. Please install Node.js." -ForegroundColor Red
        exit 1
    }
} catch {
    Write-Host "✗ Node.js not found. Please install Node.js." -ForegroundColor Red
    exit 1
}

# Check Rust
try {
    $rustResult = & rustc --version 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✓ Rust found: $rustResult" -ForegroundColor Green
    } else {
        Write-Host "✗ Rust not found. Please install Rust." -ForegroundColor Red
        exit 1
    }
} catch {
    Write-Host "✗ Rust not found. Please install Rust." -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "All prerequisites found!" -ForegroundColor Green
Write-Host ""

# Step 1: Setup PostgreSQL
Write-Host "Step 1: Setting up PostgreSQL database..." -ForegroundColor Yellow
Write-Host "Please enter your PostgreSQL credentials (or press Enter for defaults):"
$pgUser = Read-Host "PostgreSQL user [orbit]"
if ([string]::IsNullOrWhiteSpace($pgUser)) { $pgUser = "orbit" }
$pgPass = Read-Host "PostgreSQL password [orbit]"
if ([string]::IsNullOrWhiteSpace($pgPass)) { $pgPass = "orbit" }
$pgHost = Read-Host "PostgreSQL host [localhost]"
if ([string]::IsNullOrWhiteSpace($pgHost)) { $pgHost = "localhost" }

Write-Host "Creating database and user..."
$createDbSql = @"
CREATE USER $pgUser WITH PASSWORD '$pgPass';
CREATE DATABASE orbit OWNER $pgUser;
GRANT ALL PRIVILEGES ON DATABASE orbit TO $pgUser;
"@

$createDbSql | & psql -U postgres -h $pgHost 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ Database setup complete" -ForegroundColor Green
} else {
    Write-Host "⚠ Database may already exist or there was an error. Continuing..." -ForegroundColor Yellow
}

Write-Host ""

# Step 2: Configure Backend
Write-Host "Step 2: Configuring backend..." -ForegroundColor Yellow
$backendDir = Join-Path $PSScriptRoot "backend"
$envFile = Join-Path $backendDir ".env"

if (Test-Path $envFile) {
    Write-Host "⚠ .env file already exists. Skipping configuration." -ForegroundColor Yellow
} else {
    $databaseUrl = "postgres://$pgUser`:$pgPass@$pgHost`:5432/orbit"
    $jwtSecret = -join ((48..57) + (65..90) + (97..122) | Get-Random -Count 32 | % {[char]$_})
    
    @"
RUST_LOG=info
DATABASE_URL=$databaseUrl
REDIS_URL=redis://localhost:6379
JWT_SECRET=$jwtSecret
BIND_ADDRESS=127.0.0.1:8080
ACCESS_TOKEN_TTL_SECS=86400
SESSION_TTL_HOURS=12
STUN_SERVER=stun:stun.l.google.com:19302
TURN_SERVER=
CORS_ALLOWED_ORIGINS=
"@ | Out-File -FilePath $envFile -Encoding UTF8
    
    Write-Host "✓ Backend .env file created" -ForegroundColor Green
    Write-Host "  Database URL: $databaseUrl"
    Write-Host "  JWT Secret: $jwtSecret"
}

Write-Host ""

# Step 3: Build Backend
Write-Host "Step 3: Building backend..." -ForegroundColor Yellow
Set-Location $backendDir
& cargo build 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ Backend built successfully" -ForegroundColor Green
} else {
    Write-Host "✗ Backend build failed" -ForegroundColor Red
    exit 1
}
Write-Host ""

# Step 4: Build Chrome Extension
Write-Host "Step 4: Building Chrome extension..." -ForegroundColor Yellow
$extensionDir = Join-Path $PSScriptRoot "chrome-extension"
Set-Location $extensionDir

if (!(Test-Path "node_modules")) {
    Write-Host "Installing npm dependencies..."
    & npm install 2>&1
}

& npm run build 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ Chrome extension built successfully" -ForegroundColor Green
    Write-Host "  Load the 'dist' folder in Chrome at chrome://extensions/" -ForegroundColor Cyan
} else {
    Write-Host "✗ Chrome extension build failed" -ForegroundColor Red
    exit 1
}
Write-Host ""

# Step 5: Build Desktop Agent
Write-Host "Step 5: Building desktop agent..." -ForegroundColor Yellow
$agentDir = Join-Path $PSScriptRoot "desktop-agent"
Set-Location $agentDir
& cargo build 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ Desktop agent built successfully" -ForegroundColor Green
} else {
    Write-Host "✗ Desktop agent build failed" -ForegroundColor Red
    exit 1
}
Write-Host ""

# Step 6: Start Backend
Write-Host "Step 6: Starting backend server..." -ForegroundColor Yellow
Write-Host "The backend will run in a new window. Close it when done testing." -ForegroundColor Cyan
$backendScript = @"
cd '$backendDir'
cargo run
"@
Start-Process powershell -ArgumentList "-NoExit", "-Command", $backendScript

# Wait for backend to start
Write-Host "Waiting for backend to start..."
Start-Sleep -Seconds 5

# Test backend health
try {
    $response = Invoke-WebRequest -Uri "http://127.0.0.1:8080/health" -UseBasicParsing -TimeoutSec 5
    if ($response.StatusCode -eq 200) {
        Write-Host "✓ Backend is running and healthy" -ForegroundColor Green
    }
} catch {
    Write-Host "⚠ Backend may not be responding. Check the backend window." -ForegroundColor Yellow
}

Write-Host ""

# Step 7: Instructions
Write-Host "=== Setup Complete! ===" -ForegroundColor Green
Write-Host ""
Write-Host "Next Steps:" -ForegroundColor Cyan
Write-Host "1. Load the Chrome extension from chrome-extension/dist folder"
Write-Host "2. Register/login in the extension"
Write-Host "3. Register a device and copy the device token"
Write-Host "4. Configure desktop-agent/.env with the device token"
Write-Host "5. Run the desktop agent: cd desktop-agent && cargo run"
Write-Host "6. Connect to your device from the extension"
Write-Host ""
Write-Host "Press any key to open the desktop agent configuration guide..."
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")

$agentEnvFile = Join-Path $agentDir ".env"
if (!(Test-Path $agentEnvFile)) {
    @"
ORBIT_BACKEND_HOST=127.0.0.1:8080
ORBIT_DEVICE_TOKEN=
ORBIT_USER_TOKEN=
ORBIT_DEVICE_NAME=
ORBIT_SESSION_ID=
ORBIT_ICE_SERVERS=stun:stun.l.google.com:19302
ORBIT_CAPTURE_WIDTH=1920
ORBIT_CAPTURE_HEIGHT=1080
ORBIT_MIN_FPS=30
ORBIT_MAX_FPS=60
"@ | Out-File -FilePath $agentEnvFile -Encoding UTF8
    Write-Host "✓ Desktop agent .env file created" -ForegroundColor Green
}

notepad $agentEnvFile

Write-Host ""
Write-Host "After configuring the agent, run it with:" -ForegroundColor Cyan
Write-Host "cd desktop-agent && cargo run"
Write-Host ""
Write-Host "Happy testing!" -ForegroundColor Green
