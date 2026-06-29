# AgentPay Contracts

Soroban smart contracts for the AgentPay protocol: escrow, usage recording, and payment settlement on Stellar.

## Overview

- **escrow** — Records usage and supports settlement logic for machine-to-machine payments.

## Documentation

- [CHANGELOG](CHANGELOG.md) — versioned history of entrypoints, events, and error codes; contribution conventions.
- [EscrowError code table](docs/escrow/errors.md) — full reference for all 17 error codes: trigger conditions, overloaded codes, and the entrypoints that raise each code.

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

### Admin proposal validation

`propose_admin_transfer` rejects proposing the current admin as the new admin
(panics with `InvalidAdminProposal`). This surfaces no-op handovers as caller
mistakes rather than silently storing a pending entry equal to the active admin.

### Per-agent rate limiting (fixed window)

`record_usage` supports an optional per-agent rate limit anchored to
`env.ledger().timestamp()`. It is configured by two admin settings and is
**disabled by default** (both default to `0`):

- `set_max_requests_per_window(max)` — max `requests` an agent may accumulate
  per window (`get_max_requests_per_window`).
- `set_rate_window_seconds(seconds)` — the **fixed** window length
  (`get_rate_window_seconds`).

The limiter is active only when **both** are non-zero. Semantics are a
**fixed window** (not sliding): the window opens at an agent's first in-window
call and rolls forward as a whole once `now >= window_start + window_seconds`,
resetting the count. A call that would push the in-window count above the cap
is rejected with `RateLimitExceeded` (#15). State is per-agent
(`DataKey::RateWindow(agent)`), and an agent can never reset its own window
early — `window_start` only advances. Window arithmetic is saturating.

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

| Command                      | Description           |
| ---------------------------- | --------------------- |
| `cargo fmt --all`            | Format code           |
| `cargo fmt --all -- --check` | Check formatting (CI) |
| `cargo build`                | Build                 |
| `cargo test`                 | Run tests             |

## Documentation

- [Escrow: Build, Test, and Deploy Guide](docs/escrow/build-deploy.md) — build the release WASM, run the test suite, and deploy to testnet with the Stellar/Soroban CLI.
- [Escrow: Schema Versioning & Migration](docs/escrow/migrations.md) — the difference between `version()` and `SchemaVersion`, the double-run guard, and the migration runbook.
- [record_usage validation precedence](docs/escrow/validation-order.md) — numbered reference table of all 9 validation gates in `record_usage`: the error each raises (with code), whether its read is conditional, and the rationale for its position in the chain.

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

### Agent authorization on `record_usage`

`record_usage` now requires the recorded `agent` to authorize the call via
`agent.require_auth()`. This closes a usage-forgery vector where any party
could inflate a competitor agent's counters — and therefore its bill on the
next `settle` — with no signature from the agent.

#### Validation chain position

Auth is checked at **step 0**, before the pause gate:

| Step | Check                  | Error                          |
| ---- | ---------------------- | ------------------------------ |
| 0    | `agent.require_auth()` | Soroban host auth error        |
| 1    | Contract paused        | `#4 ContractPaused`            |
| 2    | `requests == 0`        | `#2 RequestsMustBePositive`    |
| 3    | `requests > max`       | `#8 RequestsExceedsMaxPerCall` |
| 4    | `requests < min`       | `#9 RequestsBelowMinPerCall`   |
| 5    | Service not registered | `#7 ServiceNotRegistered`      |
| 6    | Service disabled       | `#12 ServiceDisabled`          |
| 7    | Agent not allowed      | `#10 AgentNotAllowed`          |

#### Operator override (metering loop migration)

Soroban's auth tree supports sub-invocation authorization — an agent can
pre-authorize a trusted metering operator to call `record_usage` on its
behalf by having the operator's call appear as a sub-invocation of an
agent-signed outer call. This means existing off-chain settlement loops
can continue to operate without requiring every agent to sign each
individual `record_usage` call directly, as long as the operator is
authorized via the auth tree.

**Migration path for existing metering operators:**

1. The agent signs an outer transaction that authorizes the operator's
   contract call via Soroban's `authorize_as_current_contract` or
   sub-invocation auth.
2. The operator's metering loop submits `record_usage` as a
   sub-invocation within that authorized context.
3. Alternatively, agents can sign each `record_usage` call directly
   (standard path) if the metering loop supports it.
