# Escrow Schema Versioning and Migration Runbook

This document describes how the AgentPay escrow contract separates the compiled
contract version from the persisted storage schema version, and how operators
should run the current v1 to v2 migration.

## Versioning model

The escrow contract exposes two related but different version signals:

| Signal | Source | Meaning |
| --- | --- | --- |
| `version()` | Compiled contract code | Returns the wasm/API compatibility version. The current code returns `2`. |
| `get_schema_version()` | Persistent storage slot `DataKey::SchemaVersion` | Returns the on-chain storage schema version. If the slot is absent, the contract treats it as schema `1`. |

The distinction matters during redeployments. A new wasm can report
`version() == 2` immediately after it is deployed, while the ledger state it
reads may still be v1 until `migrate_v1_to_v2()` stamps
`DataKey::SchemaVersion` with `2`.

## Why absent v2 slots are safe

The v2 code reads every new storage slot with a conservative default, so the
contract can keep serving existing state before the schema marker is written:

| Storage key | Reader default when absent | Operational effect |
| --- | --- | --- |
| `SchemaVersion` | `1` | A legacy deployment is treated as v1 until migration runs. |
| `Paused` | `false` | Existing state-changing calls are not paused by default. |
| `RequireServiceRegistration` | `false` | Existing services continue to accept usage records unless strict registration is enabled. |
| `MaxRequestsPerCall` | `u32::MAX` | Existing callers keep the old "no cap" behavior. |
| `MinRequestsPerCall` | `0` | Existing callers keep the old "no floor" behavior. |
| `AllowlistEnabled` | `false` | Existing agents are not blocked until the allowlist gate is enabled. |
| `AgentAllowed(agent)` | `false` | Only matters after `AllowlistEnabled` is set to true. |
| `TotalUsageByAgent(agent)` | `0` | Historical lifetime analytics start from zero for the new counter. |
| `TotalRequestsAllTime` | `0` | Protocol-wide analytics start from zero for the new counter. |
| `LastSettlement(agent, service)` | `None` | Unsettled or pre-v2 pairs remain distinguishable from a real timestamp. |
| `ServiceMetadata(service)` | `None` | Services without metadata continue to work. |
| `ServiceDisabled(service)` | `false` | Existing services are not disabled by default. |

Because these defaults are explicit in the v2 readers, the migration does not
need to fan out over historical usage records. It only records that the schema
has been acknowledged as v2.

## Migration guardrails

`migrate_v1_to_v2(env)` is intentionally narrow:

1. It loads `DataKey::Admin` and fails with `EscrowError::NotInitialized (#3)`
   if `init` has not been called.
2. It requires admin authorization with `admin.require_auth()`.
3. It reads `DataKey::SchemaVersion`, defaulting an absent value to `1`.
4. It writes `DataKey::SchemaVersion = 2` only when the current value is `1`.
5. It fails with `EscrowError::MigrationVersionMismatch (#11)` when the current
   schema is anything other than `1`.

The `MigrationVersionMismatch (#11)` guard is the double-run protection. A
second `migrate_v1_to_v2()` call sees schema `2` and fails instead of silently
rewriting the marker. Future migrations should keep the same pattern so
operators can distinguish "already migrated" from "migration completed now".

## Operator runbook

Use this sequence whenever upgrading an initialized v1 escrow deployment to the
current v2 code:

1. Deploy or redeploy the v2 escrow wasm according to the standard Soroban
   deployment process for the target network.
2. Confirm the code version:
   ```bash
   soroban contract invoke \
     --id <ESCROW_CONTRACT_ID> \
     --source <ADMIN_IDENTITY> \
     --network <NETWORK> \
     -- version
   ```
   The expected result for this release is `2`.
3. Check the current schema marker:
   ```bash
   soroban contract invoke \
     --id <ESCROW_CONTRACT_ID> \
     --source <ADMIN_IDENTITY> \
     --network <NETWORK> \
     -- get_schema_version
   ```
   A legacy v1 deployment returns `1` because the schema slot is absent.
4. Run the migration from the current admin identity:
   ```bash
   soroban contract invoke \
     --id <ESCROW_CONTRACT_ID> \
     --source <ADMIN_IDENTITY> \
     --network <NETWORK> \
     -- migrate_v1_to_v2
   ```
5. Verify the stored schema marker:
   ```bash
   soroban contract invoke \
     --id <ESCROW_CONTRACT_ID> \
     --source <ADMIN_IDENTITY> \
     --network <NETWORK> \
     -- get_schema_version
   ```
   The expected result after a successful migration is `2`.
6. Optionally run a read-only smoke check for representative services and
   agents:
   - `get_usage(agent, service)` still returns the pre-migration usage total.
   - `get_total_requests_all_time()` returns `0` until v2 records new usage.
   - `is_paused()` returns `false` unless an admin explicitly paused the
     contract.

Do not run `migrate_v1_to_v2()` from a non-admin identity. The function is
admin-gated and will require the stored admin's signature.

## Forward migration convention

Future schema upgrades should be append-only:

- Add a new `migrate_v2_to_v3()`-style entrypoint instead of changing
  `migrate_v1_to_v2()` semantics.
- Keep the previous migration callable but guarded by its exact source schema.
- Assign new `EscrowError` codes only at the end of the enum so client SDKs keep
  stable numeric meanings.
- Make every new storage reader define an explicit default before migration, or
  document why a migration must backfill the value.
- Update this runbook with the new source schema, target schema, verification
  commands, and any manual backfill steps.

This keeps operator runbooks auditable: each migration has one source version,
one target version, and one verification call.
