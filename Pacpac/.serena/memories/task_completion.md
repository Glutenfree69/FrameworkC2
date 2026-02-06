# What to do when a task is completed

1. **Build**: Run `make` to verify compilation succeeds
2. **Check size**: Verify `stub.exe` stays small (target: < 30 KB final, < 10 KB for step 1)
3. **Check IAT**: `x86_64-w64-mingw32-objdump -x stub.exe | grep -A 20 'Import'` — should be empty
4. **Check strings**: `x86_64-w64-mingw32-strings stub.exe` — no leaked API names
5. **Test on Windows**: The stub needs to be tested on a Windows x64 machine
