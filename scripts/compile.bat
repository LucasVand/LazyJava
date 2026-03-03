@echo off

echo Cleaning old build...
if exist build rmdir /s /q build
mkdir build

echo Collecting Java sources...

set "SOURCES="

for /R %%f in (*.java) do (
    set "SOURCES=!SOURCES! "%%f""
)

if "%SOURCES%"=="" (
    echo No Java files found.
    exit /b 1
)

echo Compiling...
javac -d build %SOURCES%

if %errorlevel% neq 0 (
    echo.
    echo Build failed.
    exit /b %errorlevel%
)

echo.
echo Build successful!
pause
