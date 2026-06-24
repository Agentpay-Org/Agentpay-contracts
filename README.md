# AgentPay Contracts

Soroban smart contracts for the AgentPay protocol: escrow, usage recording, and payment settlement on Stellar.

## Overview

- **escrow** — Records usage and supports settlement logic for machine-to-machine payments.

### Service ownership handover

A service's `ServiceMetadata` carries a `description` and an `owner`. The
current owner (or the admin) can reassign the `owner` via
`transfer_service_ownership(caller, service_id, new_owner)` without touching the
`description`. The call honours the pause gate and emits `owner_chg` for
indexers.
### Service metadata vs. registration

A service's metadata (`description` + `owner`) and its registration flag live in
independent storage slots. `clear_service_metadata` (admin-gated, idempotent)
removes only the metadata; the registration flag and per-(agent, service) usage
history are untouched.
### Service pricing: removed vs. set-to-zero

`set_service_price` stores a per-request price under
`DataKey::ServicePrice(service_id)`. `remove_service_price` (admin-gated,
honours the pause gate, idempotent) deletes that slot and emits `price_rm`.
After removal, `get_service_price` and `compute_billing` read back `0`, exactly
as for a service that was never priced. The zero-vs-removed distinction is about
storage, not the read value: removal frees the storage slot (and emits
`price_rm`), whereas `set_service_price(service_id, 0)` leaves a stored slot
holding `0`. Both cases bill to zero, but only removal reclaims the slot.

### Admin proposal validation

`propose_admin_transfer` rejects proposing the current admin as the new admin
(panics with `InvalidAdminProposal`). This surfaces no-op handovers as caller
mistakes rather than silently storing a pending entry equal to the active admin.

### Pricing requires registration (strict mode)

`set_service_price` is coupled to the same `RequireServiceRegistration` flag that
`record_usage` honours. When strict mode is **off** (the default), pricing any
`service_id` is allowed — fully backward compatible. When it is **on**
(`set_require_service_registration(true)`), a price can only attach to a
registered service; pricing an unregistered one panics with `ServiceNotRegistered`
(#7). A **disabled** service is always rejected with `ServiceDisabled` (#12),
mirroring `record_usage`. On success, `set_service_price` emits a
`price_set(service_id, price_stroops)` event after every validation passes.
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

## Documentation

- [Escrow: Build, Test, and Deploy Guide](docs/escrow/build-deploy.md) — build the release WASM, run the test suite, and deploy to testnet with the Stellar/Soroban CLI.
- [Escrow: Entrypoint & Error-Code Reference](docs/escrow/api.md) — every entrypoint with its signature, auth/pause requirements, and panics, plus the full `EscrowError` catalogue.

## CI/CD

On push/PR to `main`, GitHub Actions runs:

- Format check (`cargo fmt --all -- --check`)
- Build (`cargo build`)
- Tests (`cargo test`)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for the full guide, including the
append-only error-code table, event conventions, and the test/coverage gate.

1. Fork the repo and create a branch.
2. Make changes; ensure `cargo fmt`, `cargo build`, and `cargo test` pass locally.
3. Open a pull request. CI must pass before merge.

## License

MIT
