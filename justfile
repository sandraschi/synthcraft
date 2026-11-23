set shell := ["powershell.exe", "-NoProfile", "-Command"]

default:
    @just --list

test:
    cargo test --workspace

clippy:
    cargo clippy --workspace --all-targets -- -D warnings

smoke:
    ./start.ps1 -Smoke

floor:
    ./scripts/craft-floor.ps1

# Build and stage dist/ (plain exes, installer, launcher zip, SHA256SUMS). Nothing is published.
release-dry:
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts/release.ps1

# Same, then tag, push the tag and upload to GitHub (clean, pushed trunk required).
release:
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts/release.ps1 -Publish
