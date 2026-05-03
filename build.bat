@echo off
chcp 65001 >nul
echo ========================================
echo   BOBOTexture V2 - Windows Build
echo ========================================
echo.
cd /d "%~dp0"

echo [1/4] Installing dependencies...
call npm install --silent
if %ERRORLEVEL% neq 0 (
    echo ERROR: npm install failed
    pause
    exit /b 1
)

echo.
echo [2/4] Building frontend + Rust backend...
call npm run tauri:build
if %ERRORLEVEL% neq 0 (
    echo ERROR: build failed
    pause
    exit /b 1
)

echo.
echo [3/4] Committing changes...
git add -A
for /f "delims=" %%i in ('git status --porcelain') do set HAS_CHANGES=1
if defined HAS_CHANGES (
    git commit -m "auto: build %date% %time%"
    echo Committed.
) else (
    echo Nothing to commit.
)

echo.
echo [4/4] Pushing to GitHub...
git push origin main
if %ERRORLEVEL% neq 0 (
    echo WARNING: git push failed (check network)
)

echo.
echo ========================================
echo   BUILD COMPLETE
echo ========================================
echo.
echo Output: src-tauri\target\release\bobotexture-v2.exe
echo.
pause
