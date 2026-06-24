# Escrow — Schema Versioning & Migration

The escrow contract tracks **two** independent version numbers. Confusing
them is the most common operational mistake, so this document defines both
and gives a migration runbook.

## Two version numbers

| | `version()` | `get_schema_version()` |
|---|---|---|
| What it is | the compiled contract (WASM) version | the persisted on-chain storage layout |
| Where it lives | hard-coded in the code (currently `2`) | `DataKey::SchemaVersion` in persistent storage |
| Default when absent | n/a (always returns a constant) | `1` (the implicit pre-migration layout) |
| Changes when | you deploy new code | you run a `migrate_*` entrypoint |

A fresh `init` stamps `SchemaVersion = 2` directly, so a contract that was
*initialised* on v2 code never needs to migrate. Migration only matters
when a **v1-era** persisted state is served by **v2** code after a code
upgrade/redeploy.

## Why v2 reads default sensibly

Every v2 read defaults gracefully when its slot is absent:

- counters and prices default to `0`,
- boolean flags default to `false` (via `read_flag`),
- `MaxRequestsPerCall` defaults to `u32::MAX`, `MinRequestsPerCall` to `0`,
- `get_last_settlement` / `get_service_metadata` return `None`.

Because of this, the v1→v2 migration body has **no data fan-out**: it only
stamps the new `SchemaVersion`. All new slots simply read their defaults
until written.

## The double-run guard

`migrate_v1_to_v2` reads the current `SchemaVersion` (defaulting to `1`)
and panics with `MigrationVersionMismatch` (#11) if it is not exactly `1`.
This makes a second run — or a run against a freshly-initialised v2
contract — fail loudly instead of silently corrupting the version stamp.
Migration is admin-gated (`require_auth` on the stored admin).

## Runbook

1. Deploy / redeploy the v2 WASM over the existing contract id.
2. Confirm the starting state: `get_schema_version()` returns `1`.
3. As the admin, call `migrate_v1_to_v2()`.
4. Verify: `get_schema_version()` now returns `2`.
5. A repeat call must panic with `Error(Contract, #11)` — expected and safe.

## Forward path

Future migrations follow the same shape and the append-only convention:
add a `migrate_v2_to_v3` that guards on `SchemaVersion == 2`, performs any
fan-out, then stamps `3`. Never renumber an existing schema version or
reuse a migration entrypoint name.

See [api.md](api.md) for the full entrypoint and error-code reference.
