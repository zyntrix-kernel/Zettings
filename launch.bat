@echo off
rem Launches Zettings desktop application detached in the background
cd /d "%~dp0build\windows\x64\runner\Release"
start "" "zettings.exe"
