@echo off
chcp 65001 >nul
echo ========================================
echo   BOBOTexture V2 - Windows Build
echo ========================================
echo.
cd /d "%~dp0"

echo [1/3] Installing dependencies...
call npm install --silent
if %ERRORLEVEL% neq 0 (
    echo ERROR: npm install failed
    pause
    exit /b 1
)

echo.
echo [2/3] Building frontend + Rust backend...
call npm run tauri:build
if %ERRORLEVEL% neq 0 (
    echo ERROR: build failed
    pause
    exit /b 1
)

echo.
echo ========================================
echo   BUILD COMPLETE
echo ========================================
echo.
echo Output: src-tauri\target\release\bobotexture-v2.exe
echo.
pause
