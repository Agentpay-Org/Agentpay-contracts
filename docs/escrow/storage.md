# Escrow Contract — Storage DataKey Reference

This document is the authoritative map of every `DataKey` variant used by the
escrow contract (`contracts/escrow/src/lib.rs`). It describes what each key
stores, its value type, what `unwrap_or` default is used when absent, which
entrypoints write it, and whether it is a lifetime counter or can be drained by
`settle`.

---

## Why everything is `persistent()`

Soroban offers three storage tiers — `instance`, `temporary`, and `persistent`.

- **`instance`** is tied to the contract instance's TTL; it disappears when the
  contract is evicted.
- **`temporary`** has an independent, short TTL and is designed for ephemeral
  state.
- **`persistent`** has an independent, configurable TTL that can be extended; it
  survives contract eviction and is appropriate for state that must outlive any
  single transaction cycle.

The escrow contract stores everything in `persistent()` because:

1. Usage and settlement counters (`Usage`, `TotalUsageByAgent`,
   `TotalRequestsAllTime`, `TotalSettledByAgent`, `TotalSettledAllTime`) must
   survive between the moment usage is recorded and the moment the off-chain
   settlement loop reads and drains them — a window that can span many ledger
   TTL cycles.
2. Configuration singletons (`Admin`, `Paused`, `SchemaVersion`, rate-limit
   settings) must be available at every call; losing them on eviction would
   brick the contract.
3. Per-service and per-agent flags (`ServiceRegistered`, `AgentAllowed`, etc.)
   are operational state, not ephemeral hints — they must survive indefinitely
   until explicitly removed by an admin entrypoint.

---

## Key cardinality

| Category | Cardinality | Notes |
|---|---|---|
| Singletons | O(1) | One slot per key type, regardless of services or agents |
| Per-service | O(S) | One slot per registered `service_id` Symbol |
| Per-agent | O(A) | One slot per unique agent `Address` |
| Per-(agent, service) pair | O(A × S) | One slot per unique `(agent, service_id)` combination |

In typical deployments the number of services S is small (tens to hundreds) and
is admin-controlled. The per-agent and per-pair cardinality grows with protocol
usage and drives the rent footprint. Off-chain settlement loops must drain
per-pair counters regularly to keep storage costs bounded.

---

## DataKey Reference Table

### Singletons

| DataKey variant | Value type | Default when absent | Written by | Drained by `settle`? |
|---|---|---|---|---|
| `Admin` | `Address` | — (must exist after `init`) | `init`, `accept_admin_transfer` | No — lifetime |
| `PendingAdmin` | `Address` | `None` (Option) | `propose_admin_transfer` | No — removed by `accept_admin_transfer` or `cancel_admin_transfer` |
| `Paused` | `bool` | `false` | `pause`, `unpause` | No — lifetime |
| `SchemaVersion` | `u32` | `1` (implicit v1) | `init`, `migrate_v1_to_v2` | No — lifetime |
| `RequireServiceRegistration` | `bool` | `false` | `set_require_service_registration` | No — lifetime |
| `MaxRequestsPerCall` | `u32` | `u32::MAX` (no cap) | `set_max_requests_per_call` | No — lifetime |
| `MinRequestsPerCall` | `u32` | `0` (no floor) | `set_min_requests_per_call` | No — lifetime |
| `AllowlistEnabled` | `bool` | `false` | `set_allowlist_enabled` | No — lifetime |
| `MaxRequestsPerWindow` | `u32` | `0` (limiter disabled) | `set_max_requests_per_window` | No — lifetime |
| `WindowSeconds` | `u64` | `0` (limiter disabled) | `set_rate_window_seconds` | No — lifetime |
| `TotalRequestsAllTime` | `u64` | `0` | `record_usage` | No — lifetime (never reset) |
| `TotalSettledAllTime` | `i128` (stroops) | `0` | `settle`, `settle_all` | No — lifetime (never reset) |
| `UsageAlertThreshold` | `u32` | `0` (alerting disabled) | *(none — read only by `record_usage`; see note below)* | No — lifetime |
| `MinServicePrice` | `i128` (stroops) | `0` (no floor) | `set_price_bounds` | No — lifetime |
| `MaxServicePrice` | `i128` (stroops) | `i128::MAX` (no ceiling) | `set_price_bounds` | No — lifetime |

### Per-service slots — cardinality O(S)

| DataKey variant | Key parameter | Value type | Default when absent | Written by | Drained by `settle`? |
|---|---|---|---|---|---|
| `ServicePrice(service_id)` | `Symbol` | `i128` (stroops) | `0` (free/unset) | `set_service_price`; removed by `remove_service_price` | No — lifetime |
| `ServiceRegistered(service_id)` | `Symbol` | `bool` | `false` | `register_service`, `register_service_with_metadata`; removed by `unregister_service` | No — lifetime |
| `ServiceDisabled(service_id)` | `Symbol` | `bool` | `false` | `set_service_disabled` | No — lifetime |
| `ServiceMetadata(service_id)` | `Symbol` | `ServiceMetadata { description: String, owner: Address }` | `None` (Option) | `set_service_metadata`, `register_service_with_metadata`, `transfer_service_ownership`; removed by `clear_service_metadata` | No — lifetime |
| `PriceTiers(service_id)` | `Symbol` | `Vec<PriceTier>` | `None` (Option) — flat `ServicePrice` used instead | `set_price_tiers`; removed by `remove_price_tiers` | No — lifetime |

### Per-agent slots — cardinality O(A)

| DataKey variant | Key parameter | Value type | Default when absent | Written by | Drained by `settle`? |
|---|---|---|---|---|---|
| `AgentAllowed(agent)` | `Address` | `bool` | `false` | `set_agent_allowed` | No — lifetime |
| `AgentBlocked(agent)` | `Address` | `bool` | `false` | `set_agent_blocked` | No — lifetime |
| `TotalUsageByAgent(agent)` | `Address` | `u32` | `0` | `record_usage` | No — lifetime (never reset by `settle`) |
| `TotalSettledByAgent(agent)` | `Address` | `i128` (stroops) | `0` | `settle`, `settle_all` | No — lifetime (never reset by `settle`) |
| `RateWindow(agent)` | `Address` | `(u64, u32)` = `(window_start, count)` | `(0, 0)` | `record_usage` (rate-limit path) | No — rolls forward on next call when window expires |
| `AgentCredit(agent)` | `Address` | `i128` (stroops) | `0` | `credit_agent`; drawn down by `debit_agent_credit` (called from `record_usage`) | No — drawn down on debit, not on `settle` |
| `AgentServiceIndex(agent)` | `Address` | `Vec<Symbol>` | `[]` (empty) | `index_agent_service` (called from `record_usage`); trimmed by `deindex_agent_service` | No — entries removed only when a service's usage is fully deindexed |
| `AgentServices(agent)` | `Address` | `Vec<Symbol>` | — never read or written | *(unused)* | N/A |

### Per-(agent, service) pair slots — cardinality O(A × S)

| DataKey variant | Key parameters | Value type | Default when absent | Written by | Drained by `settle`? |
|---|---|---|---|---|---|
| `Usage(agent, service_id)` | `Address`, `Symbol` | `u32` | `0` | `record_usage` | **Yes** — reset to `0` by `settle` |
| `LastSettlement(agent, service_id)` | `Address`, `Symbol` | `u64` (ledger timestamp, seconds since Unix epoch) | `None` (Option) | `settle` | No — stamped (not cleared) by `settle` |
| `Dispute(agent, service_id)` | `Address`, `Symbol` | `bool` | `false` | `open_dispute`; cleared by `resolve_dispute` | No — cleared on resolution, not on `settle` |

---

## Persistent-storage model details

### `Usage(agent, service_id)`

The primary accumulator. Every `record_usage(agent, service_id, requests)` call
adds `requests` to the existing counter via saturating addition. `settle` reads
the current value, computes `usage * price_stroops`, resets the slot to `0`, and
stamps `LastSettlement`. This is the **only** key drained by `settle`.

### `TotalUsageByAgent(agent)` vs `Usage(agent, service_id)`

`TotalUsageByAgent` is a cross-service lifetime counter. `settle` does **not**
touch it — it accumulates forever (saturating at `u32::MAX`). It is intended for
analytics and SLA tiering, not for billing. The per-pair `Usage` counter is the
billing source of truth.

### `TotalSettledByAgent(agent)` and `TotalSettledAllTime`

These counters track lifetime settled value in stroops. `settle` and
`settle_all` add each non-zero billed amount via saturating arithmetic and never
subtract from the counters. `get_total_settled_by_agent(agent)` and
`get_total_settled_all_time()` return `0` when the corresponding slot is absent.
Unlike `Usage`, these counters are never drained by settlement and are intended
for credit limits, loyalty pricing, and protocol-level settled-value analytics
without replaying historical `settled` events.

### `RateWindow(agent)` — fixed-window semantics

Stores `(window_start: u64, count: u32)`. On each `record_usage` call (when the
limiter is active):

1. If `now >= window_start + window_seconds`, the window rolls: `window_start =
   now`, `count = 0`.
2. `count` is incremented by `requests` (saturating).
3. If the new `count > MaxRequestsPerWindow`, the call is rejected.
4. Otherwise the updated `(window_start, count)` is persisted.

An agent can never reset its own window early — `window_start` only advances.

### `AgentCredit(agent)` — prepaid balance

Set via `credit_agent(agent, amount)` (admin-gated, rejects non-positive
`amount`). Drawn down inside `record_usage` by `debit_agent_credit`, which
subtracts the newly-billed amount and panics with
`InsufficientCreditBalance` if the draw would go negative. There is no public
debit entrypoint — the only way this slot decreases is as a side effect of
`record_usage` succeeding.

### `AgentServiceIndex(agent)` and the unused `AgentServices(agent)`

`AgentServiceIndex(agent)` is the real per-agent index of services with
non-zero recorded usage. It backs `get_agent_services`,
`get_agent_usage_page`, `list_open_disputes`, and `settle_all`'s iteration.
`index_agent_service` appends a `service_id` the first time usage is
recorded for a pair; `deindex_agent_service` removes it once the pair's
usage returns to `0`.

`AgentServices(agent)` is a separate `DataKey` variant that is declared but
never read or written anywhere in `lib.rs` — despite an inline comment
elsewhere describing it as "the alias used by `settle_all`". `settle_all`
actually reads `AgentServiceIndex` directly (see the code above). Treat
`AgentServices` as dead storage until it is wired up or removed; do not rely
on it holding a value.

### `UsageAlertThreshold` — configured but not yet settable

`record_usage` reads this key to decide whether to emit a `usage_hi` alert
event on the crossing edge (see [`docs/escrow/events.md`](events.md)), but no
entrypoint in the current contract writes it. In practice the slot is always
absent, so `record_usage` always uses the `unwrap_or(0)` default and the
alerting block is permanently skipped. This is a gap, not a design choice —
an admin setter is needed before this feature can be exercised on-chain.

### `MinServicePrice` / `MaxServicePrice` — global price bounds

Set together via `set_price_bounds(min_stroops, max_stroops)` (admin-gated).
`set_service_price` reads both (defaulting to `0` and `i128::MAX` when
absent) and rejects any price outside `[MinServicePrice, MaxServicePrice]`
with `PriceOutOfBounds`. See
[`docs/escrow/pricing.md`](pricing.md#global-price-bounds) for the full
interaction, including the important caveat that `set_price_tiers` does
**not** consult these bounds.

### `Dispute(agent, service_id)`

`true` while a dispute is open for the pair. `open_dispute` (caller = the
agent) sets it and rejects a second open with `DisputeAlreadyOpen`.
`resolve_dispute` (admin only) clears it after adjusting the usage counter.
**Invariant gap:** `settle` never reads this flag — an open dispute does not
stop the pair from being drained and billed by `settle` or `settle_all`.
Off-chain settlement tooling must consult `has_open_dispute` /
`list_open_disputes` itself before settling a disputed pair; the contract
does not enforce the hold on-chain.

### `LastSettlement(agent, service_id)`

Stores the ledger timestamp at which `settle` last drained this pair. Returns
`None` for pairs never settled (distinct from `Some(0)`, which would imply a
genesis-block settlement). Off-chain SLA monitors use this to detect stuck
settlement cycles.

### Schema version

`SchemaVersion` stores a `u32` schema version number, distinct from the compiled
WASM `version()`. It tracks what the persisted state layout looks like. A fresh
v2 `init` stamps `2` directly; a contract migrated from v1 gets `2` written by
`migrate_v1_to_v2`. Reading `get_schema_version()` returns `1` (the implicit
default) for pre-migration contracts.

---

## Security notes

- **`Usage` is the only drained key.** All other keys are either lifetime
  singletons, per-service/per-agent flags, or monotonically growing counters.
  Settlement accounting relies on `Usage` starting at `0` after each `settle`
  call; any code path that writes `Usage` outside of `record_usage` and `settle`
  would break billing invariants.
- **Lifetime counters are never reset.** `TotalUsageByAgent`,
  `TotalRequestsAllTime`, `TotalSettledByAgent`, and `TotalSettledAllTime`
  must not be treated as settlement-cycle deltas by downstream analytics.
- **Settled-value counters are monotonic.** Non-positive bills leave
  `TotalSettledByAgent` and `TotalSettledAllTime` unchanged, and positive bills
  use saturating addition so overflow clamps at `i128::MAX`.
- **`AgentBlocked` takes precedence over `AgentAllowed`.** An agent that is both
  blocked and allow-listed is rejected. Implementations relying on the allowlist
  gate must ensure the blocklist is not populated with the same address.
- **Per-pair cardinality drives rent.** A large population of `(agent,
  service_id)` pairs with unsettled usage will accumulate storage rent. The
  off-chain settlement loop should drain pairs regularly to bound persistent
  storage costs.
- **`Dispute` does not gate `settle`.** See the note above — a caller with
  settlement rights can still drain a disputed pair. Do not rely on
  `open_dispute` alone as an on-chain hold.
- **`AgentServices` is dead storage.** The variant exists but nothing reads
  or writes it; the real per-agent index is `AgentServiceIndex`.
- **`UsageAlertThreshold` cannot currently be set.** No entrypoint writes it,
  so the `usage_hi` alert path in `record_usage` is unreachable until a
  setter is added.
