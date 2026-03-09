@echo off
cd /d "%~dp0"
setlocal EnableDelayedExpansion
title AetherLens Lab Launcher

:MENU
cls
echo ========================================================
echo   AETHERLENS - CYBERSECURITY LAB PLATFORM
echo ========================================================
echo.
echo  [1] Start Full Lab (Live Capture Mode)
echo      * Requires Administrator Privileges (for Npcap)
echo      * Captures real network traffic
echo.
echo  [2] Start Demo Mode (Simulation)
echo      * Safe mode, no Admin required
echo      * Generates synthetic attack traffic
echo      * Best for testing UI and features
echo.
echo  [3] Install/Update Dependencies
echo      * Runs npm install, cargo build, dotnet restore
echo.
echo  [4] Docker Compose Up (Demo Mode)
echo      * Runs everything in containers
echo.
echo  [0] Exit
echo.
set /p choice="Select an option (0-4): "

if "%choice%"=="1" goto START_LIVE
if "%choice%"=="2" goto START_DEMO
if "%choice%"=="3" goto INSTALL_DEPS
if "%choice%"=="4" goto DOCKER_UP
if "%choice%"=="0" exit /b

echo Invalid choice.
pause
goto MENU

:START_LIVE
cls
echo [SYSTEM] Starting LIVE Mode (Admin required for packet capture)...
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo [ERROR] Administrator privileges required for Live Mode.
    echo Please right-click and "Run as Administrator".
    pause
    goto MENU
)
set DEMO_MODE=false
set CARGO_FEATURES=--features packet-capture
if not defined NPCAP_SDK_DIR (
    if exist "%~dp0npcap-sdk\Lib\x64\Packet.lib" (
        set "NPCAP_SDK_DIR=%~dp0npcap-sdk\Lib\x64"
        echo [SYSTEM] Found Npcap SDK at: !NPCAP_SDK_DIR!
    )
)
goto LAUNCH_ALL

:START_DEMO
cls
echo [SYSTEM] Starting DEMO Mode (Simulation)...
set DEMO_MODE=true
set CARGO_FEATURES=
goto LAUNCH_ALL

:LAUNCH_ALL
echo.
echo [0/3] Cleaning up old processes and ports...
taskkill /F /IM aether_core.exe >nul 2>&1
taskkill /F /IM api-server.exe >nul 2>&1
echo   - Checking for processes on Port 5000...
powershell -Command "Get-NetTCPConnection -LocalPort 5000 -ErrorAction SilentlyContinue | ForEach-Object { Stop-Process -Id $_.OwningProcess -Force }" >nul 2>&1
echo   - Checking for processes on Port 50051...
powershell -Command "Get-NetTCPConnection -LocalPort 50051 -ErrorAction SilentlyContinue | ForEach-Object { Stop-Process -Id $_.OwningProcess -Force }" >nul 2>&1
echo.

echo [1/3] Launching Rust Core Engine...
start "AetherLens Core (Rust)" cmd /k "cd core-engine && set DEMO_MODE=%DEMO_MODE%&& cargo run --release %CARGO_FEATURES%"
timeout /t 5 /nobreak >nul

echo [2/3] Launching .NET Backend API...
start "AetherLens Backend (.NET)" cmd /k "cd api-server && dotnet run"
timeout /t 3 /nobreak >nul

echo [3/3] Launching Web Dashboard...
start "AetherLens Dashboard" cmd /k "cd web-dashboard && npm run dev"

echo.
echo [SUCCESS] System is running!
echo Dashboard: http://localhost:3000
echo Backend:   http://localhost:5000
echo.
pause
goto MENU

:INSTALL_DEPS
cls
echo [SYSTEM] Installing dependencies...
echo.
echo [1/3] Rust Core...
cd core-engine
cargo build
cd ..
echo.
echo [2/3] .NET Backend...
cd api-server
dotnet restore
cd ..
echo.
echo [3/3] Web Dashboard...
cd web-dashboard
call npm install
cd ..
echo.
echo [SUCCESS] All dependencies installed.
pause
goto MENU

:DOCKER_UP
cls
echo [SYSTEM] Starting Docker containers...
docker compose up --build -d
echo.
echo [SUCCESS] Containers started (Demo Mode).
echo Dashboard: http://localhost:3000
pause
goto MENU
