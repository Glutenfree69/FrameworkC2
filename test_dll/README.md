# Test DLL pour PE Loader

## Compilation

```bash
# Depuis macOS avec MinGW
x86_64-w64-mingw32-gcc -shared -o test_dll.dll test_dll.c -luser32

# Chiffrement XOR (même clé que le loader)
python3 -c "
key = b'Fr4m3w0rkC2_K3y!'
with open('test_dll.dll', 'rb') as f:
    data = f.read()
encrypted = bytes(b ^ key[i % len(key)] for i, b in enumerate(data))
with open('test_dll.dll.enc', 'wb') as f:
    f.write(encrypted)
print(f'Encrypted {len(data)} bytes -> test_dll.dll.enc')
"
```

## Utilisation

1. Compiler la DLL
2. Chiffrer avec XOR
3. Remplacer `evil.dll.enc` par `test_dll.dll.enc` dans le loader
4. Tester sur Windows

## Différence avec ReflectiveLdr

Cette DLL est une DLL **standard** Windows :
- Pas besoin de `ReflectiveLoader` export
- Le PE Loader mappe la DLL en mémoire
- Le PE Loader appelle directement `DllMain`

La DLL `evil.dll` avec ReflectiveLdr ne fonctionne PAS avec le nouveau PE Loader car :
- Elle contient son propre loader interne
- Elle s'attend à être appelée via `ReflectiveLoader`, pas `DllMain`
