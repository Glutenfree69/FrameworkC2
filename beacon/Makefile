# Makefile - Beacon C

CC = x86_64-w64-mingw32-gcc
CFLAGS = -Wall -Wextra -O2
LIBS = -lwinhttp -lws2_32 -lole32

SRC_DIR = src
SOURCES = $(SRC_DIR)/main.c \
          $(SRC_DIR)/sysinfo.c \
          $(SRC_DIR)/json.c \
          $(SRC_DIR)/http.c \
          $(SRC_DIR)/commands.c

OUTPUT = beacon.dll
OUTPUT_DEBUG = beacon_debug.dll

all: $(OUTPUT)

$(OUTPUT): $(SOURCES)
	$(CC) $(CFLAGS) -shared -o $@ $^ $(LIBS)
	@echo "✅ Compilation réussie: $(OUTPUT)"

debug: CFLAGS += -DDEBUG -g
debug: $(SOURCES)
	$(CC) $(CFLAGS) -shared -o $(OUTPUT_DEBUG) $^ $(LIBS)
	@echo "✅ Compilation debug réussie: $(OUTPUT_DEBUG)"

test-exe: CFLAGS += -DDEBUG -g
test-exe: $(SOURCES)
	$(CC) $(CFLAGS) -o beacon_test.exe $^ $(LIBS) -mconsole
	@echo "✅ Test exe compilé: beacon_test.exe"

clean:
	rm -f $(OUTPUT) $(OUTPUT_DEBUG) beacon_test.exe

.PHONY: all debug test-exe clean