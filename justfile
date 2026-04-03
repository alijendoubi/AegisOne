# AegisOne — development task runner
# Install just: cargo install just  |  brew install just  |  scoop install just
#
# Usage:
#   just          → list all tasks
#   just dev      → start web dev server
#   just agent    → build agent in debug mode
#   just test     → run all tests (web + agent)
#   just check    → lint + typecheck everything without building

set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# ── Meta ───────────────────────────────────────────────────────────────────────

# List available tasks (default)
default:
    @just --list

# ── Web ────────────────────────────────────────────────────────────────────────

# Start the Next.js dev server
dev:
    cd web && npm run dev

# Install web dependencies
web-install:
    cd web && npm ci

# Type-check the web app
web-typecheck:
    cd web && npm run typecheck

# Lint the web app
web-lint:
    cd web && npm run lint

# Production build of the web app
web-build:
    cd web && npm run build

# Run all web checks (lint + typecheck + build)
web-ci: web-install web-lint web-typecheck web-build

# ── Agent ──────────────────────────────────────────────────────────────────────

# Build the agent in debug mode
agent:
    cd agent && cargo build

# Build the agent in release mode
agent-release:
    cd agent && cargo build --release

# Run agent unit + integration tests
agent-test:
    cd agent && cargo test --all-targets

# Clippy lint the agent (deny warnings)
agent-lint:
    cd agent && cargo clippy --all-targets -- -D warnings

# Check formatting
agent-fmt-check:
    cd agent && cargo fmt --check

# Auto-format agent code
agent-fmt:
    cd agent && cargo fmt

# Run all agent checks (fmt + clippy + test)
agent-ci: agent-fmt-check agent-lint agent-test

# Run the agent in dry-run watch mode (no destructive actions)
agent-watch-dry:
    cd agent && cargo run -- watch --dry-run

# Install canary files (requires the agent to be built)
agent-canary-install:
    cd agent && cargo run -- canary install

# Verify canary files are intact
agent-canary-verify:
    cd agent && cargo run -- canary verify

# List VSS shadow copies (Windows only, requires elevation)
agent-snapshots:
    cd agent && cargo run -- snapshot list

# Print the resolved config (useful for debugging defaults)
agent-config-show:
    cd agent && cargo run -- config show

# Write the default config to the platform config directory
agent-config-init:
    cd agent && cargo run -- config init

# ── Combined ───────────────────────────────────────────────────────────────────

# Run all checks across the whole repo
check: web-lint web-typecheck agent-fmt-check agent-lint
    @echo "All checks passed."

# Run all tests across the whole repo
test: agent-test
    @echo "All tests passed."

# Full CI equivalent (what GitHub Actions runs)
ci: web-ci agent-ci
    @echo "Full CI passed."

# ── Supabase ───────────────────────────────────────────────────────────────────

# Apply the waitlist migration to a local Supabase instance
db-migrate:
    supabase db push

# Reset the local Supabase database and re-apply migrations
db-reset:
    supabase db reset

# ── Utilities ──────────────────────────────────────────────────────────────────

# Clean all build artifacts
clean:
    cd agent && cargo clean
    cd web && rm -rf .next out

# Show the current git status
status:
    git status --short
    git log --oneline -5
