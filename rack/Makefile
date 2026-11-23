# VCV Rack plugin build for SynthCraft.
#
#   1. Build the engine:   cargo build --release -p synthcraft-ffi      (from the repo root)
#   2. Build the plugin:   make RACK_DIR=/path/to/Rack-SDK
#   3. Install:            make install RACK_DIR=/path/to/Rack-SDK
#
# FFI_LIB points at the static library from step 1. Rust's standard library needs a few
# system libraries at link time; the per-OS list below is what `rustc --print native-static-libs`
# reports for the supported targets. On Windows build the engine for the MinGW target Rack's
# toolchain uses (`--target x86_64-pc-windows-gnu`) and point FFI_LIB at that library.

RACK_DIR ?= ../../Rack-SDK
FFI_LIB ?= ../target/release/libsynthcraft_ffi.a

FLAGS += -I../crates/synthcraft-ffi/include

SOURCES += $(wildcard src/*.cpp)
DISTRIBUTABLES += res

include $(RACK_DIR)/arch.mk

# The Rust static library follows the objects on the link line so the linker resolves the sc_*
# symbols from it, and the system libraries Rust needs follow the library.
LDFLAGS += $(FFI_LIB)
ifdef ARCH_LIN
	LDFLAGS += -Wl,--exclude-libs,ALL -ldl -lpthread -lm
endif
ifdef ARCH_MAC
	LDFLAGS += -framework Security -framework CoreFoundation -lpthread -lm
endif
ifdef ARCH_WIN
	LDFLAGS += -lws2_32 -luserenv -lbcrypt -lntdll -lkernel32 -ladvapi32
endif

include $(RACK_DIR)/plugin.mk
