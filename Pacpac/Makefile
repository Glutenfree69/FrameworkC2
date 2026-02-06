# =============================================
# Makefile — Packer C stub, no-CRT, Windows x64
# Cross-compilation depuis macOS/Linux avec MinGW
# =============================================

CC      = x86_64-w64-mingw32-gcc
STRIP   = x86_64-w64-mingw32-strip

CFLAGS  = -Os -nostdlib -fno-stack-protector -fno-ident \
          -fpack-struct=8 -Wall -Wextra \
          -Iinclude

LDFLAGS = -nostdlib -Wl,-eEntry -Wl,--no-seh \
          -Wl,-s -Wl,--subsystem,console

SRCS    = src/entry.c src/peb.c src/utils.c src/chacha20.c src/lz4.c src/tls.c src/loader.c
OUT     = stub.exe

all: $(OUT)

$(OUT): $(SRCS)
	$(CC) $(CFLAGS) $(LDFLAGS) -o $@ $(SRCS)
	$(STRIP) --strip-all $@
	@ls -la $@
	@echo "Done. Check IAT with: x86_64-w64-mingw32-objdump -x stub.exe | grep -A 20 'Import'"

clean:
	rm -f $(OUT)

.PHONY: all clean
