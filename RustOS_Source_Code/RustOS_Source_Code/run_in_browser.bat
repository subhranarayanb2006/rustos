@echo off
title RustOS Web Browser Runner
echo ========================================================
echo   Starting RustOS Live Web Browser Emulator (v86)
echo ========================================================
echo.
echo [1/2] Opening http://localhost:8080 in your web browser...
start http://localhost:8080
echo.
echo [2/2] Local Web Server is running on port 8080.
echo       Keep this window open while using RustOS!
echo       (Press Ctrl+C when you want to stop the server)
echo.
if exist .tools\python\python.exe (
    .tools\python\python.exe -m http.server 8080 --directory huggingface
) else (
    python -m http.server 8080 --directory huggingface
)
pause
