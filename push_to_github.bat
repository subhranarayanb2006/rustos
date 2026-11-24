@echo off
title Push RustOS to GitHub
echo ========================================================
echo          Upload RustOS Project to GitHub
echo ========================================================
echo.

:: Check for Git
where git >nul 2>&1
if %ERRORLEVEL% equ 0 (
    set GIT_CMD=git
) else if exist ".tools\git\cmd\git.exe" (
    set GIT_CMD=".tools\git\cmd\git.exe"
) else (
    echo [ERROR] Git was not found in your system PATH.
    echo Please install Git from https://git-scm.com or add it to PATH.
    pause
    exit /b 1
)

echo [1/4] Checking Git repository status...
if not exist ".git" (
    echo Initializing fresh Git repository...
    %GIT_CMD% init
)

echo.
echo [2/4] Staging all project files and Windows GUI assets...
%GIT_CMD% add .

echo.
echo [3/4] Committing changes...
%GIT_CMD% commit -m "feat: RustOS 32-bit Operating System with Windows 95/XP Desktop & Unix Shell"

echo.
echo [4/4] Configuring GitHub Remote Repository...
echo.
set /p REPO_URL="Enter your GitHub repository URL (e.g. https://github.com/subhranarayanb2006/rustos.git): "

if "%REPO_URL%"=="" (
    echo No URL entered. Skipping push.
    echo Your changes are committed locally!
    pause
    exit /b 0
)

:: Set remote
%GIT_CMD% remote remove origin >nul 2>&1
%GIT_CMD% remote add origin %REPO_URL%
%GIT_CMD% branch -M main

echo.
echo Pushing to GitHub (%REPO_URL%)...
%GIT_CMD% push -u origin main

if %ERRORLEVEL% equ 0 (
    echo.
    echo ========================================================
    echo      SUCCESS! RustOS is now live on GitHub!
    echo ========================================================
) else (
    echo.
    echo [NOTE] If GitHub asked for authentication:
    echo 1. Generate a Personal Access Token (PAT) at:
    echo    https://github.com/settings/tokens
    echo 2. Use your GitHub username and the PAT as your password.
)

echo.
pause
