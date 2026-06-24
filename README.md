# AgentPay Contracts

Soroban smart contracts for the AgentPay protocol: escrow, usage recording, and payment settlement on Stellar.

## Overview

- **escrow** — Records usage and supports settlement logic for machine-to-machine payments.

### Schema version: fresh v2 init vs. legacy v1→v2 migration

`init` stamps the current storage schema version (v2) directly, so a freshly
deployed contract reports `get_schema_version() == 2` without ever running a
migration. A legacy contract deployed before this change carries the implicit v1
default and must call `migrate_v1_to_v2()` to reach v2; calling that migration on
a fresh v2 deploy panics with `MigrationVersionMismatch`.

## Prerequisites

- [Rust](https://rustup.rs/) (stable, with `rustfmt`)
- [Stellar Soroban CLI](https://soroban.stellar.org/docs) (optional, for deployment)

## Setup for contributors

1. **Clone the repo** (or add remote and pull):
   ```bash
   git clone <repo-url> && cd agentpay-contracts
   ```

2. **Install Rust** (if needed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup component add rustfmt
   ```

3. **Verify setup**:
   ```bash
   cargo fmt --all -- --check
   cargo build
   cargo test
   ```

## Project structure

```
agentpay-contracts/
├── Cargo.toml              # Workspace root
├── contracts/
│   └── escrow/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs      # Contract logic
│           └── test.rs     # Unit tests
└── .github/workflows/
    └── ci.yml              # CI: fmt, build, test
```

## Commands

| Command | Description |
|--------|-------------|
| `cargo fmt --all` | Format code |
| `cargo fmt --all -- --check` | Check formatting (CI) |
| `cargo build` | Build |
| `cargo test` | Run tests |

## CI/CD

On push/PR to `main`, GitHub Actions runs:

- Format check (`cargo fmt --all -- --check`)
- Build (`cargo build`)
- Tests (`cargo test`)

## Contributing

1. Fork the repo and create a branch.
2. Make changes; ensure `cargo fmt`, `cargo build`, and `cargo test` pass locally.
3. Open a pull request. CI must pass before merge.

## License

MIT
