# SynthCraft for VCV Rack

A polyphonic synthesizer module on the SynthCraft engine. Build, ports, behaviour and licence notes are in `../docs/PLUGINS.md`.

Quick build (Linux, macOS or Windows with MinGW), from this directory:

```
cargo build --release -p synthcraft-ffi     # run in the repo root first
make RACK_DIR=/path/to/Rack-SDK
make install RACK_DIR=/path/to/Rack-SDK
```

Status: compiled and linked against Rack SDK 2.6.4 on Linux; not yet loaded in a running Rack.
