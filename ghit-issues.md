---
type: Feature
title: "Settle escrow invoices on-chain via Stellar Asset Contract (SAC) token transfers"
labels: type:feature, area:settlement, stack:soroban, stack:rust, priority:high, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Implement on-chain settlement via Stellar Asset Contract (SAC) token transfers

### Description
Today `settle()` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) computes the outstanding bill (`accumulated_requests × price_per_request`), resets the usage counter to zero, emits a `settled` event, and **returns the billed amount** — but it intentionally **holds no balance and moves no funds**. Actual value transfer is delegated to an off-chain settlement loop, which means the on-chain record and the real payment can drift apart and there is no trustless guarantee that an agent's debt was ever paid. This issue closes that gap by moving a configurable **SAC token** from the agent's pre-funded escrow balance to the service owner, atomically with the counter reset.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add a configurable settlement token: store a `token: Address` (SAC contract id) set once by admin at `init` or via a dedicated admin entrypoint, persisted under a new `DataKey::SettlementToken`.
- Add `deposit(agent, amount)` so an agent pre-funds an on-chain escrow balance (`DataKey::Balance(Address)`), using `token::Client::transfer` with `agent.require_auth()`.
- Modify `settle(agent, service_id)` so that, after computing `billed`, it transfers `min(billed, balance)` from the agent's escrow balance to the service owner (resolved from `ServiceMetadata.owner`), debits the balance, then resets the usage counter and stamps `LastSettlement`.
- Add a typed error (e.g. `InsufficientEscrowBalance`) so under-funded settlements fail loudly. Keep error codes **append-only** to preserve client-SDK stability.
- Preserve all existing invariants: pause gate, admin `require_auth`, saturating arithmetic, and the `settled` event (extend its payload with the transferred amount).

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feature/contracts-01-sac-onchain-settlement`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — new `DataKey::SettlementToken` / `DataKey::Balance(Address)`, `set_settlement_token`, `deposit`, `withdraw`, and the `settle()` transfer logic via `soroban_sdk::token`.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — use `env.register_stellar_asset_contract` / a mock token, asserting balance deltas, partial-settlement failure, and event payloads.
  - **Add documentation:** update [`README.md`](README.md) and add `docs/escrow/settlement.md` describing the deposit → record → settle → withdraw lifecycle.
  - Include NatSpec-style doc comments (`///`) on every new entrypoint, matching the existing style in `lib.rs`.
  - Validate security assumptions: no fund lock-up, no double-spend on repeated `settle`, correct `require_auth`, and overflow safety on balance math.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases and failure paths: zero balance, exact-balance settlement, under-funded settlement, paused contract, unregistered/disabled service, and unauthorized caller.
- Include the full `cargo test` output and a short **security notes** section in the PR description (threat model + mitigations).

### Example commit message
`feat: settle escrow invoices on-chain via SAC token transfers with tests and docs`

### Guidelines
- **Minimum 95 percent test coverage** for impacted modules.
- Clear, reviewer-focused documentation.
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord for questions, reviews, and faster merges:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — if this issue and the maintainers helped you ship, we'd be grateful for a **5-star rating**. Clear questions in Discord and tidy, well-tested PRs are the fastest path to a merge and a reward.
++++++
---
type: Feature
title: "Add a reset_usage entrypoint to drain an agent-service counter without settling"
labels: type:feature, area:usage-accounting, stack:soroban, stack:rust, priority:high, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Implement the missing reset_usage entrypoint referenced in the contract docs

### Description
The doc comment on `unregister_service` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) tells callers to "call reset_usage or remove the price separately if a clean wipe is required" — but **no `reset_usage` entrypoint exists** in the contract. This is a documented-but-missing API: there is no admin-gated way to zero a `(agent, service_id)` counter without running `settle` (which also stamps `LastSettlement` and emits a billing event). This issue implements the entrypoint the docs already promise.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add `reset_usage(env, agent, service_id)`: admin-gated via the existing `DataKey::Admin` + `require_auth()` pattern, honouring the `Paused` gate like `settle`.
- Zero `DataKey::Usage(agent, service_id)` without touching `LastSettlement`, `TotalUsageByAgent`, or `TotalRequestsAllTime` — distinguish a "wipe" from a "settlement" in the audit trail.
- Emit a distinct event (e.g. `("usage_reset",)`) carrying `(agent, service_id, prev_total)` so off-chain monitors can tell a reset from a settle.
- Update the `unregister_service` doc comment to link the now-real entrypoint, and document `reset_usage` in [`README.md`](README.md).

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feature/contracts-02-reset-usage`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — new `reset_usage` entrypoint reusing the admin-auth and pause-gate helpers.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — assert the counter zeroes, lifetime counters are untouched, the event fires, and unauthorized/paused calls panic.
  - **Add documentation:** update [`README.md`](README.md) and the `unregister_service` doc comment.
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: only admin can wipe, no silent loss of lifetime analytics.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: never-used pair (no-op), paused contract, non-admin caller, and verifying `get_total_usage_by_agent` is preserved.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`feat: add reset_usage entrypoint to drain a counter without settling`

### Guidelines
- **Minimum 95 percent test coverage** for impacted modules.
- Clear, reviewer-focused documentation.
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord for questions, reviews, and faster merges:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — if this issue and the maintainers helped you ship, we'd be grateful for a **5-star rating**. Clear questions in Discord and tidy, well-tested PRs are the fastest path to a merge and a reward.
++++++
---
type: Feature
title: "Enforce the documented 256-byte service-description cap in set_service_metadata"
labels: type:bug, area:service-registry, stack:soroban, stack:rust, priority:high, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Harden set_service_metadata to enforce the documented description length cap

### Description
The doc comment on `set_service_metadata` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) states the description "is capped at 256 UTF-8 bytes to bound storage cost" — but the function body performs **no length check** and writes the `String` straight into `DataKey::ServiceMetadata`. This is a documentation/behaviour mismatch and an unbounded-storage griefing vector: a malicious or buggy admin tool can write arbitrarily large descriptions, inflating ledger rent. This issue makes the code match its contract.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- In `set_service_metadata`, read `description.len()` (byte length on the Soroban `String`) and reject values above 256 with a new typed error (e.g. `DescriptionTooLong`), appended to `EscrowError` to keep codes stable.
- Decide and document the exact boundary semantics (256 allowed, 257 rejected) and assert it in tests.
- Confirm the cap is enforced before any storage write so an over-long value never lands on-chain.
- Update the doc comment to reference the error and keep the "bound storage cost" rationale.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b bug/contracts-03-enforce-description-cap`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — length guard in `set_service_metadata` + new `EscrowError` variant.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — exactly-256 accepted, 257 rejected with the new error code, empty description accepted.
  - **Add documentation:** clarify the cap and error in [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: storage-cost griefing closed, no panic on valid input.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: boundary lengths (255/256/257), multi-byte UTF-8 characters near the cap, and non-admin caller.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`fix: enforce documented 256-byte description cap in set_service_metadata`

### Guidelines
- **Minimum 95 percent test coverage** for impacted modules.
- Clear, reviewer-focused documentation.
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord for questions, reviews, and faster merges:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — if this issue and the maintainers helped you ship, we'd be grateful for a **5-star rating**. Clear questions in Discord and tidy, well-tested PRs are the fastest path to a merge and a reward.
++++++
---
type: Feature
title: "Emit structured events on admin, price, and registration state changes"
labels: type:feature, area:events, stack:soroban, stack:rust, priority:high, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Add observability events for admin, pricing, and registry mutations

### Description
Only `record_usage`, `settle`, and `pause`/`unpause` publish events in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs). Admin-sensitive mutations — `set_service_price`, `register_service`/`unregister_service`, `propose_admin_transfer`/`accept_admin_transfer`, `set_agent_allowed`, `set_service_disabled`, `migrate_v1_to_v2` — change critical state **silently**, leaving indexers and security monitors blind to governance actions. This issue adds a consistent event for every administrative state change.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Publish a topic-tagged event from each admin mutation: e.g. `price_set(service_id, price)`, `svc_reg`/`svc_unreg(service_id)`, `admin_prop(new_admin)`, `admin_rot(old, new)`, `allow_set(agent, allowed)`, `svc_dis(service_id, disabled)`, `migrated(from, to)`.
- Use `symbol_short!` topics (≤9 chars) and consistent data tuples so a single subscriber schema can decode them.
- Do not change any existing event payloads except where another issue explicitly extends them; keep this purely additive.
- Document the full event catalogue in a new `docs/escrow/events.md`.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feature/contracts-04-admin-events`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — `env.events().publish(...)` calls in each admin entrypoint.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — assert event topics/data via `env.events().all()` for each mutation.
  - **Add documentation:** add `docs/escrow/events.md` and link it from [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: events must not leak more than the corresponding state already exposes.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: idempotent writes still emit, and topic lengths stay within Soroban symbol limits.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`feat: emit structured events on admin, price, and registration changes`

### Guidelines
- **Minimum 95 percent test coverage** for impacted modules.
- Clear, reviewer-focused documentation.
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord for questions, reviews, and faster merges:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — if this issue and the maintainers helped you ship, we'd be grateful for a **5-star rating**. Clear questions in Discord and tidy, well-tested PRs are the fastest path to a merge and a reward.
++++++
---
type: Feature
title: "Add batch record_usage to amortise ledger write cost across many services"
labels: type:feature, area:usage-accounting, stack:soroban, stack:rust, priority:high, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Implement a batched record_usage entrypoint

### Description
`record_usage` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) records exactly one `(agent, service_id, requests)` tuple per invocation. A busy agent calling many services within a metering window must pay per-transaction overhead for each, which is expensive on Soroban where each call has fixed ledger costs. This issue adds a batch entrypoint that records many usage deltas in a single transaction while preserving every existing validation.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add `record_usage_batch(env, agent, entries: Vec<(Symbol, u32)>)` returning a `Vec<UsageRecord>` of new totals.
- Apply all existing gates per entry: pause, zero-request rejection, min/max per-call bounds, strict-registration, service-disabled, and the allowlist.
- Decide and document atomicity: the batch should be all-or-nothing — if any entry fails validation the whole call panics so partial state is never persisted.
- Bound the batch length to prevent unbounded-loop gas griefing (e.g. a `MaxBatchSize` constant or admin-configurable cap with a typed error).

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feature/contracts-05-batch-record-usage`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — `record_usage_batch` reusing the validation logic factored out of `record_usage`.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — multi-entry success, mid-batch failure rolls back, lifetime counters sum correctly.
  - **Add documentation:** document the batch API and atomicity guarantee in [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: bounded loop, saturating arithmetic preserved, single event vs. per-entry events documented.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: empty batch, oversized batch, duplicate service ids in one batch, and a failing entry mid-batch.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`feat: add batched record_usage to amortise ledger write cost`

### Guidelines
- **Minimum 95 percent test coverage** for impacted modules.
- Clear, reviewer-focused documentation.
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord for questions, reviews, and faster merges:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — if this issue and the maintainers helped you ship, we'd be grateful for a **5-star rating**. Clear questions in Discord and tidy, well-tested PRs are the fastest path to a merge and a reward.
++++++
---
type: Feature
title: "Introduce role-based access control beyond the single-admin model"
labels: type:feature, area:access-control, stack:soroban, stack:rust, priority:high, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Implement role-based access control for operator and pauser roles

### Description
Every privileged entrypoint in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) gates on the single `DataKey::Admin` address. This concentrates pricing, registration, settlement, pausing, and migration under one key — operationally fragile and a single point of compromise. This issue introduces a minimal role system so day-to-day operations (pausing, pricing) can be delegated without handing over full admin authority.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add a `Role` enum (e.g. `Admin`, `Operator`, `Pauser`) and `DataKey::Role(Address)` mapping; admin can grant/revoke roles.
- Map entrypoints to required roles: `pause`/`unpause` → Pauser or Admin; `set_service_price`/`register_service`/`set_service_disabled` → Operator or Admin; `propose_admin_transfer`/`migrate_v1_to_v2`/role grants → Admin only.
- Add a typed `Unauthorized` error and a `has_role` read entrypoint. Preserve the two-step admin handover as the root of trust.
- Keep backward compatibility: an account that was the sole admin must retain all powers after migration.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feature/contracts-06-rbac`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — `Role` type, `grant_role`/`revoke_role`/`has_role`, and a shared `require_role` helper.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — each role can/cannot reach each entrypoint, grant/revoke round-trips.
  - **Add documentation:** add `docs/escrow/roles.md` describing the permission matrix.
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: no privilege escalation, admin remains revoke-proof for itself.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: revoking the last admin (must be prevented), unknown caller, role granted then revoked.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`feat: add role-based access control for operator and pauser roles`

### Guidelines
- **Minimum 95 percent test coverage** for impacted modules.
- Clear, reviewer-focused documentation.
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord for questions, reviews, and faster merges:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — if this issue and the maintainers helped you ship, we'd be grateful for a **5-star rating**. Clear questions in Discord and tidy, well-tested PRs are the fastest path to a merge and a reward.
++++++
---
type: Feature
title: "Add contract upgradeability via update_current_contract_wasm"
labels: type:feature, area:upgradeability, stack:soroban, stack:rust, priority:high, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Implement an admin-gated contract upgrade entrypoint

### Description
The contract in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) tracks a `SchemaVersion` and a compiled `version()` of 2, and the migration doc references "a redeployed contract" — but there is **no upgrade entrypoint**, so shipping a fix today means deploying a fresh contract id and losing all persisted usage and balances. This issue adds in-place upgradeability using Soroban's `update_current_contract_wasm` so state survives a logic upgrade.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add `upgrade(env, new_wasm_hash: BytesN<32>)`: admin-gated, `require_auth`, calling `env.deployer().update_current_contract_wasm(new_wasm_hash)`.
- Emit an `upgraded(new_wasm_hash)` event and, where appropriate, pair the upgrade with a `migrate_*` step so `SchemaVersion` advances in lockstep.
- Document the upgrade-then-migrate runbook and the trust assumption (admin can replace logic) prominently.
- Ensure the pause gate and admin two-step handover interact sanely with upgrades (e.g. recommend pausing before upgrading).

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feature/contracts-07-upgradeability`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — `upgrade` entrypoint + event.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — register a second wasm, upgrade, assert state preserved and new logic active; assert non-admin upgrade panics.
  - **Add documentation:** add `docs/escrow/upgrades.md` with the runbook.
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: only admin upgrades, no bricking path, schema/version coherence.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: upgrade while paused, upgrade by non-admin, double-upgrade, and migration after upgrade.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`feat: add admin-gated contract upgradeability with state preservation`

### Guidelines
- **Minimum 95 percent test coverage** for impacted modules.
- Clear, reviewer-focused documentation.
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord for questions, reviews, and faster merges:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — if this issue and the maintainers helped you ship, we'd be grateful for a **5-star rating**. Clear questions in Discord and tidy, well-tested PRs are the fastest path to a merge and a reward.
++++++
---
type: Feature
title: "Manage persistent-entry TTL with explicit storage bumping"
labels: type:feature, area:storage, stack:soroban, stack:rust, priority:high, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Implement TTL bumping for long-lived persistent storage entries

### Description
Every entry in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) is written via `env.storage().persistent().set(...)` with **no TTL management**. Soroban persistent entries expire and become archived once their TTL lapses; without `extend_ttl` calls, long-lived config (`Admin`, `ServicePrice`, `ServiceMetadata`) and slow-moving usage counters can be archived, breaking reads until they are restored. This issue adds explicit, consistent TTL bumping.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- On every persistent write (and ideally on hot reads of long-lived keys), call `extend_ttl` with documented threshold/extend constants tuned per key class (config vs. per-pair counters).
- Add a shared helper so the bump policy lives in one place and is easy to audit; define the `LEDGERS_*` constants near the top of the module.
- Bump the contract instance TTL where appropriate so the deployed code does not archive.
- Document the rent/TTL model and the recommended off-chain restore strategy.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feature/contracts-08-storage-ttl`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — TTL constants + a `bump_persistent` helper applied across writes.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — use `env.ledger().set` / `env.ledger().with_mut` to advance ledgers and assert entries survive past the old expiry.
  - **Add documentation:** add `docs/escrow/storage-ttl.md` explaining thresholds.
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: no unbounded write amplification from over-aggressive bumping.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: entry just under the threshold, fresh write, and instance bump.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`feat: add explicit persistent-entry TTL bumping to prevent archival`

### Guidelines
- **Minimum 95 percent test coverage** for impacted modules.
- Clear, reviewer-focused documentation.
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord for questions, reviews, and faster merges:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — if this issue and the maintainers helped you ship, we'd be grateful for a **5-star rating**. Clear questions in Discord and tidy, well-tested PRs are the fastest path to a merge and a reward.
++++++
---
type: Feature
title: "Add a dispute-and-refund flow for contested usage records"
labels: type:feature, area:disputes, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Implement a dispute and refund flow for contested settlements

### Description
The escrow in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) has no mechanism for an agent to contest a charge: `record_usage` accumulates and `settle` drains, with no path to flag, hold, or reverse a contested amount. For a machine-to-machine billing protocol this leaves no recourse for over-reporting bugs. This issue adds a lightweight dispute window and admin-adjudicated refund (built on the on-chain balance introduced by the settlement work).

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add `open_dispute(agent, service_id)` that flags a `(agent, service_id)` pair, blocking `settle` for it until resolved; persist under a new `DataKey::Dispute`.
- Add admin `resolve_dispute(agent, service_id, refund_requests)` that subtracts contested usage (or credits balance) and clears the flag, emitting a `dispute` event.
- Define and document the dispute lifecycle and which states block settlement; reuse the pause gate and admin auth patterns.
- Keep all changes additive and error codes append-only.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feature/contracts-09-dispute-refund`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — dispute storage, `open_dispute`/`resolve_dispute`, settle guard.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — open blocks settle, resolve adjusts usage, events fire, unauthorized resolve panics.
  - **Add documentation:** add `docs/escrow/disputes.md` with a state diagram.
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: no double-refund, disputes cannot be self-resolved by the agent.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: dispute on unused pair, resolve with zero refund, settle attempt during open dispute.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`feat: add dispute and refund flow for contested usage records`

### Guidelines
- **Minimum 95 percent test coverage** for impacted modules.
- Clear, reviewer-focused documentation.
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord for questions, reviews, and faster merges:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — if this issue and the maintainers helped you ship, we'd be grateful for a **5-star rating**. Clear questions in Discord and tidy, well-tested PRs are the fastest path to a merge and a reward.
++++++
---
type: Feature
title: "Add per-agent, per-window rate limiting to record_usage"
labels: type:feature, area:rate-limiting, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Implement per-agent rate limiting on usage recording

### Description
`record_usage` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) enforces per-call min/max bounds but has **no time-windowed rate limit**: a single agent can call it unboundedly within a ledger window, inflating counters and ledger write load. This issue adds a configurable per-agent rate limit anchored to `env.ledger().timestamp()` so abusive call patterns are throttled on-chain.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add admin-configurable `MaxRequestsPerWindow` and `WindowSeconds`, plus per-agent `DataKey::RateWindow(Address)` tracking `(window_start, count_in_window)`.
- In `record_usage`, roll the window forward when expired and reject calls that would exceed the cap with a new `RateLimitExceeded` error (append-only).
- Default to disabled (no limit) when unset, preserving current behaviour; expose getters for the configured values.
- Document the windowing semantics (fixed vs. sliding) precisely.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feature/contracts-10-rate-limiting`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — config keys, per-agent window state, and the rate check in `record_usage`.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — advance the ledger clock to cross windows, assert throttling and reset.
  - **Add documentation:** document the rate-limit config in [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: window math is overflow-safe and cannot be reset by the agent.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: exactly-at-cap, window rollover, disabled (default), and clock not advancing.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`feat: add per-agent per-window rate limiting to record_usage`

### Guidelines
- **Minimum 95 percent test coverage** for impacted modules.
- Clear, reviewer-focused documentation.
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord for questions, reviews, and faster merges:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — if this issue and the maintainers helped you ship, we'd be grateful for a **5-star rating**. Clear questions in Discord and tidy, well-tested PRs are the fastest path to a merge and a reward.
++++++
---
type: Feature
title: "Validate service registration before allowing set_service_price"
labels: type:enhancement, area:service-registry, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Harden set_service_price to require a registered service

### Description
`set_service_price` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) accepts a price for **any** `service_id`, even one that was never registered, has no metadata, and is disabled. This lets prices accumulate for phantom services and makes the registry and pricing tables drift apart. This issue optionally couples pricing to registration so prices can only attach to real services.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add an admin toggle (reusing the `RequireServiceRegistration` flag, or a parallel one) so that, when enabled, `set_service_price` rejects unregistered `service_id`s with `ServiceNotRegistered`.
- Optionally also reject pricing a disabled service, mirroring `record_usage`'s `ServiceDisabled` gate.
- Keep the default behaviour backward-compatible (no coupling unless the flag is on) and document the interaction with strict-registration mode.
- Emit the `price_set` event only after the validation passes.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b enhancement/contracts-11-price-requires-registration`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — registration/disabled checks in `set_service_price`.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — pricing unregistered service rejected when strict, allowed when lax.
  - **Add documentation:** clarify the coupling in [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: no behaviour change when the flag is off.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: registered+priced, unregistered+strict, disabled service, flag toggled mid-life.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`feat: optionally require service registration before set_service_price`

### Guidelines
- **Minimum 95 percent test coverage** for impacted modules.
- Clear, reviewer-focused documentation.
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord for questions, reviews, and faster merges:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — if this issue and the maintainers helped you ship, we'd be grateful for a **5-star rating**. Clear questions in Discord and tidy, well-tested PRs are the fastest path to a merge and a reward.
++++++
---
type: Feature
title: "Add a paginated query entrypoint to enumerate an agent's active services"
labels: type:feature, area:queries, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Implement queryable enumeration of an agent's service usage

### Description
Reads in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) require the caller to already know the `(agent, service_id)` pair — `get_usage` takes both. There is no way to ask "which services has this agent used?" on-chain, forcing dashboards to reconstruct the set from event logs. This issue adds a per-agent index of touched services with a bounded query entrypoint.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Maintain a per-agent `Vec<Symbol>` index of services with non-zero usage, updated in `record_usage` (and pruned on `settle`/`reset_usage` if usage hits zero, per documented policy).
- Add `get_agent_services(agent) -> Vec<Symbol>` and a `get_agent_usage_page(agent, start, limit)` returning `(Symbol, u32)` pairs to bound the response size.
- Keep the index write cost low and documented; cap the index length to avoid unbounded growth grief.
- Ensure the index stays consistent with the underlying `Usage` counters.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feature/contracts-12-agent-service-index`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — index maintenance + query entrypoints.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — index grows on new service, pagination boundaries, consistency after settle.
  - **Add documentation:** document the query API and pagination in [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: bounded index, no duplicate entries.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: empty index, single service, limit beyond end, duplicate-service recording.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`feat: add paginated agent-service enumeration query`

### Guidelines
- **Minimum 95 percent test coverage** for impacted modules.
- Clear, reviewer-focused documentation.
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord for questions, reviews, and faster merges:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — if this issue and the maintainers helped you ship, we'd be grateful for a **5-star rating**. Clear questions in Discord and tidy, well-tested PRs are the fastest path to a merge and a reward.
++++++
---
type: Feature
title: "Support per-service owner authorization for self-service settlement"
labels: type:feature, area:settlement, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Allow a service owner to settle their own service without full admin

### Description
`settle` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) requires the global admin's `require_auth()`, so the owner stored in `ServiceMetadata.owner` cannot trigger settlement for their own service — every settlement funnels through the central admin key. This issue lets the registered service owner authorize settlement for their own service, decentralising the operational load.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Modify `settle` so it accepts either the admin OR the `ServiceMetadata(service_id).owner` for that specific service, with the appropriate `require_auth()`.
- Reject settlement when no metadata/owner is set and the caller is not admin, with a clear typed error.
- Keep the pause gate and counter-drain semantics unchanged; emit the `settled` event identically.
- Document the authorization matrix (admin vs. owner) clearly.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feature/contracts-13-owner-settle`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — owner-or-admin auth branch in `settle`.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — owner settles own service, owner cannot settle another service, admin always can.
  - **Add documentation:** document the auth matrix in [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: owner of service A cannot settle service B; no metadata means admin-only.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: missing metadata, wrong owner, paused contract, admin override.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`feat: allow service owner to settle their own service`

### Guidelines
- **Minimum 95 percent test coverage** for impacted modules.
- Clear, reviewer-focused documentation.
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord for questions, reviews, and faster merges:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — if this issue and the maintainers helped you ship, we'd be grateful for a **5-star rating**. Clear questions in Discord and tidy, well-tested PRs are the fastest path to a merge and a reward.
++++++
---
type: Feature
title: "Add a tiered pricing model with volume discounts per service"
labels: type:enhancement, area:pricing, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Implement tiered, volume-discounted pricing in compute_billing

### Description
Pricing in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) is a single flat `price_per_request` via `ServicePrice`, and `compute_billing` is a plain `requests × price`. Real metered APIs offer volume discounts. This issue adds an optional tier table per service so high-usage agents are billed at progressively lower marginal rates, computed deterministically on-chain.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add an admin-set tier schedule per service: e.g. `Vec<(threshold_requests, price_stroops)>` stored under a new `DataKey::PriceTiers(Symbol)`.
- Update `compute_billing` and `settle` to apply the tier schedule when present, falling back to the flat `ServicePrice` when absent (full backward compatibility).
- Keep all math saturating and document the tier-boundary semantics (inclusive/exclusive) explicitly.
- Validate the schedule is monotonically ordered at set-time, rejecting malformed tables with a typed error.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b enhancement/contracts-14-tiered-pricing`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — `set_price_tiers`, tier-aware billing helper.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — flat fallback, multi-tier crossing, boundary requests, malformed schedule rejected.
  - **Add documentation:** add `docs/escrow/pricing.md` with worked examples.
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: overflow safety across tiers, deterministic ordering.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: usage at exact tier thresholds, empty schedule, single tier, descending malformed input.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`feat: add tiered volume-discount pricing to billing computation`

### Guidelines
- **Minimum 95 percent test coverage** for impacted modules.
- Clear, reviewer-focused documentation.
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord for questions, reviews, and faster merges:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — if this issue and the maintainers helped you ship, we'd be grateful for a **5-star rating**. Clear questions in Discord and tidy, well-tested PRs are the fastest path to a merge and a reward.
++++++
---
type: Feature
title: "Add settle_all to drain every service for an agent in one call"
labels: type:feature, area:settlement, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Implement a batched settle_all entrypoint

### Description
`settle` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) drains exactly one `(agent, service_id)` pair per call. An agent that has used many services forces the settlement loop into one transaction per service. Building on the agent-service index, this issue adds a single call to settle every outstanding service for an agent and return the per-service totals.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add `settle_all(env, agent) -> Vec<(Symbol, i128)>` that iterates the agent's active-service index and settles each, honouring pause and admin/owner auth.
- Bound the iteration to a documented maximum to keep the call within gas limits; surface a typed error on overflow of that bound.
- Emit one `settled` event per service (matching single-settle semantics) and stamp each `LastSettlement`.
- Document interaction with the dispute flow (skip disputed pairs) if that issue has landed.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feature/contracts-15-settle-all`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — `settle_all` reusing the single-settle core.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — multi-service settle, totals correct, all counters zeroed, oversized set bounded.
  - **Add documentation:** document `settle_all` in [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: bounded loop, no partial-settle inconsistency.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: agent with no usage, single service, max-bound services, paused contract.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`feat: add settle_all to drain every service for an agent in one call`

### Guidelines
- **Minimum 95 percent test coverage** for impacted modules.
- Clear, reviewer-focused documentation.
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord for questions, reviews, and faster merges:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — if this issue and the maintainers helped you ship, we'd be grateful for a **5-star rating**. Clear questions in Discord and tidy, well-tested PRs are the fastest path to a merge and a reward.
++++++
---
type: Feature
title: "Add tests for the agent allowlist enforcement path in record_usage"
labels: type:test, area:testing, stack:soroban, stack:rust, priority:high, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Test the agent allowlist gate end to end

### Description
The allowlist logic in `record_usage` (and `set_allowlist_enabled`, `set_agent_allowed`, `is_agent_allowed`) in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) has **no coverage** in [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — no test asserts that an off-allowlist agent is rejected with `AgentNotAllowed (#10)` or that disabling the gate restores access. This issue closes that gap with focused tests.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Cover: allowlist disabled (default) allows any agent; enabled + agent not listed → panic `#10`; enabled + agent allowed → succeeds; allowed then revoked → rejected again.
- Assert `is_allowlist_enabled` and `is_agent_allowed` round-trips.
- Use `#[should_panic(expected = "Error(Contract, #10)")]` matching the existing test conventions.
- Do not modify contract logic; this is a test-only change unless a genuine bug surfaces (then file separately).

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b test/contracts-16-allowlist-tests`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — no changes expected; only touch if a bug is found.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — the allowlist scenarios above.
  - **Add documentation:** note the covered behaviour in the test module header comment.
  - Include NatSpec-style doc comments (`///`) where helper functions are added.
  - Validate security: tests prove the gate cannot be bypassed.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: toggling the gate while usage exists, multiple agents with mixed status.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`test: cover agent allowlist enforcement in record_usage`

### Guidelines
- **Minimum 95 percent test coverage** for impacted modules.
- Clear, reviewer-focused documentation.
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord for questions, reviews, and faster merges:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — if this issue and the maintainers helped you ship, we'd be grateful for a **5-star rating**. Clear questions in Discord and tidy, well-tested PRs are the fastest path to a merge and a reward.
++++++
---
type: Feature
title: "Add tests for min/max per-call request bounds in record_usage"
labels: type:test, area:testing, stack:soroban, stack:rust, priority:high, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Test the per-call request floor and ceiling enforcement

### Description
`record_usage` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) rejects calls above `MaxRequestsPerCall (#8)` and below `MinRequestsPerCall (#9)`, but [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) has **no tests** for either bound or for the `set_*`/`get_*` accessors. This issue adds boundary tests so these guards are protected against regressions.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Cover: default (no cap = `u32::MAX`, no floor = 0) allows any positive value; set max then a call above it panics `#8`; set min then a call below it panics `#9`; exactly-at-bound calls succeed.
- Assert `get_max_requests_per_call` defaults to `u32::MAX` and `get_min_requests_per_call` defaults to `0`.
- Use the existing `should_panic` error-code conventions.
- Test-only change unless a genuine bug surfaces.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b test/contracts-17-per-call-bounds-tests`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — no changes expected.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — the bound scenarios above.
  - **Add documentation:** note covered behaviour in the test module header.
  - Include NatSpec-style doc comments (`///`) on any test helpers.
  - Validate security: bounds cannot be circumvented.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: min > max misconfiguration behaviour, exactly-min, exactly-max.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`test: cover min/max per-call request bounds in record_usage`

### Guidelines
- **Minimum 95 percent test coverage** for impacted modules.
- Clear, reviewer-focused documentation.
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord for questions, reviews, and faster merges:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — if this issue and the maintainers helped you ship, we'd be grateful for a **5-star rating**. Clear questions in Discord and tidy, well-tested PRs are the fastest path to a merge and a reward.
++++++
---
type: Feature
title: "Add tests for strict service-registration and service-disabled gates"
labels: type:test, area:testing, stack:soroban, stack:rust, priority:high, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Test the service-registration and service-disabled rejection paths

### Description
The `RequireServiceRegistration` gate (`ServiceNotRegistered #7`) and the `ServiceDisabled` gate (`#12`) in `record_usage` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) are uncovered by [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs). Neither `register_service`/`unregister_service` nor `set_service_disabled` is exercised against `record_usage`. This issue adds the missing coverage.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Cover: strict mode off allows unknown services; strict on + unregistered → panic `#7`; register then succeed; unregister then reject again.
- Cover: registered + disabled service → panic `#12`; re-enable then succeed; verify registration/metadata survive a disable.
- Assert `is_service_registration_required`, `is_service_registered`, and `is_service_disabled` round-trips.
- Test-only change unless a genuine bug surfaces.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b test/contracts-18-registry-gate-tests`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — no changes expected.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — the registry/disabled scenarios above.
  - **Add documentation:** note covered behaviour in the test module header.
  - Include NatSpec-style doc comments (`///`) on any test helpers.
  - Validate security: disabled services cannot accrue usage.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: disable an unregistered service, strict mode with disabled service, toggling order.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`test: cover service-registration and service-disabled gates`

### Guidelines
- **Minimum 95 percent test coverage** for impacted modules.
- Clear, reviewer-focused documentation.
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord for questions, reviews, and faster merges:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — if this issue and the maintainers helped you ship, we'd be grateful for a **5-star rating**. Clear questions in Discord and tidy, well-tested PRs are the fastest path to a merge and a reward.
++++++
---
type: Feature
title: "Add tests for lifetime usage counters and last-settlement timestamps"
labels: type:test, area:testing, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Test lifetime counters and settlement-time tracking

### Description
`TotalUsageByAgent`, `TotalRequestsAllTime`, and `LastSettlement` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) (read via `get_total_usage_by_agent`, `get_total_requests_all_time`, `get_last_settlement`) are completely untested in [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs). The contract's documented invariant — that `settle` drains per-pair usage but does **not** reset lifetime counters — is unverified. This issue locks that invariant down with tests.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Cover: cross-service accumulation into `TotalUsageByAgent`; protocol-wide `TotalRequestsAllTime` summing across agents; both survive `settle`.
- Cover: `get_last_settlement` returns `None` before any settle and `Some(timestamp)` after, using `env.ledger().with_mut` to control the clock.
- Assert the documented distinction between `None` and `Some(0)`.
- Test-only change unless a genuine bug surfaces.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b test/contracts-19-lifetime-counter-tests`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — no changes expected.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — the counter/timestamp scenarios above.
  - **Add documentation:** note covered invariants in the test module header.
  - Include NatSpec-style doc comments (`///`) on any test helpers.
  - Validate security: settlement never silently loses lifetime analytics.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: multiple agents, settle then re-record, never-settled pair.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`test: cover lifetime usage counters and last-settlement timestamps`

### Guidelines
- **Minimum 95 percent test coverage** for impacted modules.
- Clear, reviewer-focused documentation.
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord for questions, reviews, and faster merges:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — if this issue and the maintainers helped you ship, we'd be grateful for a **5-star rating**. Clear questions in Discord and tidy, well-tested PRs are the fastest path to a merge and a reward.
++++++
---
type: Feature
title: "Add tests for the two-step admin transfer and migration version guard"
labels: type:test, area:testing, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Test admin handover edge cases and schema migration guard

### Description
[`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) covers the happy-path admin rotation and the no-pending panic, but misses several edges in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs): `cancel_admin_transfer`, `accept_admin_transfer` by the wrong caller (`NotPendingAdmin #6`), re-proposing over a pending entry, and the `migrate_v1_to_v2` double-run guard (`MigrationVersionMismatch #11`). This issue adds those tests.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Cover: propose → cancel → accept fails with `#5`; propose → wrong caller accepts → `#6`; re-propose overwrites pending; `get_pending_admin` reflects each state.
- Cover: `get_schema_version` defaults to 1; `migrate_v1_to_v2` advances to 2; second migrate panics `#11`.
- Assert the rotated admin can perform an admin action and the old admin can no longer.
- Test-only change unless a genuine bug surfaces.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b test/contracts-20-admin-migration-tests`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — no changes expected.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — the handover/migration scenarios above.
  - **Add documentation:** note covered behaviour in the test module header.
  - Include NatSpec-style doc comments (`///`) on any test helpers.
  - Validate security: lockout-resistance of the two-step handover is proven.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: cancel with nothing pending, accept after cancel, migrate before any state exists.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`test: cover admin transfer edge cases and migration version guard`

### Guidelines
- **Minimum 95 percent test coverage** for impacted modules.
- Clear, reviewer-focused documentation.
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord for questions, reviews, and faster merges:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — if this issue and the maintainers helped you ship, we'd be grateful for a **5-star rating**. Clear questions in Discord and tidy, well-tested PRs are the fastest path to a merge and a reward.
++++++
---
type: Feature
title: "Add saturating-arithmetic and overflow tests for usage and billing math"
labels: type:test, area:testing, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Test saturating arithmetic in counters and billing computation

### Description
[`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) relies on `saturating_add` for usage counters and `saturating_mul` for `compute_billing`/`settle`, documented to "saturate at u32::MAX" and "i128::MAX" rather than overflow. None of these saturation edges are exercised in [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs). This issue adds tests that drive the counters and billing math to their boundaries.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Cover: per-pair usage saturating at `u32::MAX`; `TotalUsageByAgent` saturating; `TotalRequestsAllTime` (u64) accumulation near large values.
- Cover: `compute_billing` saturating at `i128::MAX` with a large price × large usage; `settle` returns the saturated value and still drains the counter.
- Set counters near the boundary by recording in large increments rather than relying on internal access.
- Test-only change unless a genuine bug surfaces.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b test/contracts-21-saturation-tests`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — no changes expected.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — the saturation scenarios above.
  - **Add documentation:** note covered invariants in the test module header.
  - Include NatSpec-style doc comments (`///`) on any test helpers.
  - Validate security: no panic/overflow under adversarial inputs.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: one-below-max then +1, exact-max, settle at saturated billing.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`test: cover saturating arithmetic in usage counters and billing`

### Guidelines
- **Minimum 95 percent test coverage** for impacted modules.
- Clear, reviewer-focused documentation.
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord for questions, reviews, and faster merges:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — if this issue and the maintainers helped you ship, we'd be grateful for a **5-star rating**. Clear questions in Discord and tidy, well-tested PRs are the fastest path to a merge and a reward.
++++++
---
type: Feature
title: "Audit and test require_auth coverage across all privileged entrypoints"
labels: type:security, area:access-control, stack:soroban, stack:rust, priority:high, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Harden and verify authorization on every state-changing entrypoint

### Description
Authorization in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) is hand-rolled per entrypoint (`admin.require_auth()` after a storage read). The test suite uses `env.mock_all_auths()`, which means **no test ever proves that a missing or wrong authorization actually fails**. This issue audits each privileged entrypoint and adds negative authorization tests using scoped auth mocking.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Enumerate every entrypoint that mutates state (`set_service_price`, `register_service`, `pause`, `propose_admin_transfer`, `set_agent_allowed`, `set_service_disabled`, `set_service_metadata`, `migrate_v1_to_v2`, `settle`, etc.) and confirm each calls `require_auth` on the correct principal before any write.
- Add tests using `env.mock_auths(&[...])` (scoped) to assert that an unauthorized caller is rejected, rather than blanket `mock_all_auths`.
- Where an entrypoint reads admin then auths, confirm ordering cannot leak a partial write on auth failure.
- Document the authorization model in a `docs/escrow/security.md` section.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b security/contracts-22-auth-audit`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — fix any entrypoint missing or mis-ordering `require_auth`.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — scoped-auth negative tests per entrypoint.
  - **Add documentation:** add `docs/escrow/security.md` with the auth matrix.
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: prove no privileged write succeeds without the correct signer.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: wrong signer, no signer, correct signer, and auth on the two-step handover principals.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`security: audit and test require_auth coverage on privileged entrypoints`

### Guidelines
- **Minimum 95 percent test coverage** for impacted modules.
- Clear, reviewer-focused documentation.
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord for questions, reviews, and faster merges:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — if this issue and the maintainers helped you ship, we'd be grateful for a **5-star rating**. Clear questions in Discord and tidy, well-tested PRs are the fastest path to a merge and a reward.
++++++
---
type: Feature
title: "Gate price and registry mutations behind the pause flag"
labels: type:security, area:access-control, stack:soroban, stack:rust, priority:high, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Harden the pause gate to cover all state-changing entrypoints

### Description
Only `record_usage` and `settle` consult the `Paused` flag in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs). Admin mutations — `set_service_price`, `register_service`/`unregister_service`, `set_service_disabled`, `set_service_metadata`, `set_agent_allowed`, the per-call bounds setters — all still execute **while the contract is paused**. A pause is meant to be a global emergency stop, so config can still drift during an incident. This issue extends the pause gate consistently.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Decide and document a clear policy: which entrypoints must respect pause (all state mutations) and which intentionally bypass it (e.g. `unpause` must always work; `propose_admin_transfer` may be argued either way).
- Add the `ContractPaused (#4)` guard to the entrypoints that should respect pause, via a shared `ensure_not_paused` helper to avoid drift.
- Keep `unpause` and read entrypoints unaffected.
- Document the matrix in `docs/escrow/security.md`.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b security/contracts-23-pause-gate-coverage`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — `ensure_not_paused` helper applied across admin mutations.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — each gated entrypoint panics `#4` while paused, and `unpause` still works.
  - **Add documentation:** document the pause matrix.
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: emergency stop truly halts state drift.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: pause then attempt each mutation, unpause still callable, reads unaffected.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`security: extend pause gate to all state-changing entrypoints`

### Guidelines
- **Minimum 95 percent test coverage** for impacted modules.
- Clear, reviewer-focused documentation.
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord for questions, reviews, and faster merges:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — if this issue and the maintainers helped you ship, we'd be grateful for a **5-star rating**. Clear questions in Discord and tidy, well-tested PRs are the fastest path to a merge and a reward.
++++++
---
type: Feature
title: "Add overflow-checked arithmetic verification under the release profile"
labels: type:security, area:arithmetic, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Harden arithmetic safety and document the overflow strategy

### Description
[`Cargo.toml`](Cargo.toml) sets `overflow-checks = true` on the release profile, while [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) deliberately uses `saturating_*` in the hot paths. The mix of "panic on overflow (checked)" and "silently saturate" is not documented and could surprise auditors — e.g. a future `+` that should saturate but doesn't, or a saturating path that masks a real bug. This issue audits every arithmetic site and makes the strategy explicit and tested.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Inventory every arithmetic operation (usage add, lifetime add, billing mul, any new balance math) and classify each as "must saturate" or "must check/panic", with a rationale comment at each site.
- Where saturation is intended, ensure it is explicit (`saturating_*`) and not relying on the build profile; where overflow must be impossible, document why.
- Add a `docs/escrow/arithmetic.md` capturing the policy and the meaning of a saturated value to downstream consumers.
- Add tests proving the chosen behaviour at each boundary.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b security/contracts-24-arithmetic-policy`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — explicit arithmetic with rationale comments.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — boundary behaviour per site.
  - **Add documentation:** add `docs/escrow/arithmetic.md`.
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: no silent wrap, saturation cannot hide accounting errors.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: max-boundary inputs for each operation, zero operands.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`security: audit and document arithmetic overflow/saturation strategy`

### Guidelines
- **Minimum 95 percent test coverage** for impacted modules.
- Clear, reviewer-focused documentation.
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord for questions, reviews, and faster merges:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — if this issue and the maintainers helped you ship, we'd be grateful for a **5-star rating**. Clear questions in Discord and tidy, well-tested PRs are the fastest path to a merge and a reward.
++++++
---
type: Feature
title: "Replace the overloaded RequestsMustBePositive error used by set_service_price"
labels: type:security, area:pricing, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Harden set_service_price to reject negative prices with a precise error

### Description
In [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs), `set_service_price` rejects a negative `price_stroops` by panicking with `EscrowError::RequestsMustBePositive (#2)` — an error whose name and doc comment are about `record_usage` request counts, not prices. This semantic mismatch will confuse client SDKs decoding the error and any operator reading logs. This issue introduces a precise error while keeping codes append-only.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add a new `EscrowError::InvalidPrice` variant (next free code, append-only) and use it for the negative-price guard in `set_service_price`.
- Do not renumber or reuse existing codes; `#2` keeps its `record_usage` meaning.
- Update the doc comment on `set_service_price` and the error enum accordingly.
- Confirm zero price remains allowed (free service) per the existing contract.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b security/contracts-25-invalid-price-error`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — new error variant + guard swap.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — negative price panics with the new code, zero allowed, positive round-trips.
  - **Add documentation:** note the error semantics in [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: no code reuse, stable SDK decoding.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: -1, i128::MIN, 0, large positive price.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`fix: use a dedicated InvalidPrice error for negative set_service_price`

### Guidelines
- **Minimum 95 percent test coverage** for impacted modules.
- Clear, reviewer-focused documentation.
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord for questions, reviews, and faster merges:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — if this issue and the maintainers helped you ship, we'd be grateful for a **5-star rating**. Clear questions in Discord and tidy, well-tested PRs are the fastest path to a merge and a reward.
++++++
---
type: Feature
title: "Document the full escrow entrypoint reference and error code catalogue"
labels: type:docs, area:docs, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Document every escrow entrypoint and EscrowError code

### Description
[`README.md`](README.md) describes the build commands and structure but contains **no API reference**: the 30+ entrypoints in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) and the 12 `EscrowError` codes are undocumented outside inline `///` comments. New integrators have no single reference for what the contract exposes or what each error means. This issue produces a complete, accurate reference.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add `docs/escrow/api.md` listing every entrypoint with signature, auth requirement, pause behaviour, parameters, return, and the errors it can panic with.
- Add an error-code table mapping each `EscrowError` variant to its numeric code, trigger condition, and the entrypoints that raise it.
- Cross-check every entry against the source so the docs cannot drift; link the new doc from [`README.md`](README.md).
- Note the append-only error-code policy so contributors do not renumber.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b docs/contracts-26-api-reference`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — only minor `///` fixes if an inline comment is found inaccurate.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — no new logic; optionally add a doc-aligned smoke assertion if a discrepancy is found.
  - **Add documentation:** add `docs/escrow/api.md` and the error table; update [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) where any are missing on public entrypoints.
  - Validate security: docs accurately state auth/pause requirements.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: ensure every public entrypoint is listed exactly once.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`docs: add complete escrow entrypoint and error-code reference`

### Guidelines
- **Minimum 95 percent test coverage** for impacted modules.
- Clear, reviewer-focused documentation.
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord for questions, reviews, and faster merges:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — if this issue and the maintainers helped you ship, we'd be grateful for a **5-star rating**. Clear questions in Discord and tidy, well-tested PRs are the fastest path to a merge and a reward.
++++++
---
type: Feature
title: "Write an integration guide for the record-settle metering lifecycle"
labels: type:docs, area:docs, stack:soroban, stack:rust, priority:low, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Document the end-to-end metering and settlement lifecycle

### Description
The README in [`README.md`](README.md) does not explain how an integrator wires up the escrow: how to `init`, register and price a service, enable the allowlist, call `record_usage`, then `compute_billing` and `settle`. The full lifecycle in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) is only discoverable by reading the source. This issue writes a practical, copy-pasteable integration guide.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add `docs/escrow/integration.md` walking through: deploy → `init(admin)` → `register_service` + `set_service_price` + `set_service_metadata` → optional allowlist/strict-registration → `record_usage` loop → `compute_billing` → `settle`.
- Include a sequence diagram and the role of each off-chain actor (agent, service owner, settlement loop).
- Show the relevant `EscrowClient` calls mirroring the patterns in [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs).
- Cross-reference the events emitted at each step so integrators know what to index.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b docs/contracts-27-integration-guide`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — only `///` clarifications if needed.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — optionally add an end-to-end lifecycle test that the guide references.
  - **Add documentation:** add `docs/escrow/integration.md`; link from [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) where gaps are found.
  - Validate security: guide highlights pause, auth, and strict-mode considerations.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: the guide's example flow compiles as a test if one is added.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`docs: add end-to-end record-settle integration guide`

### Guidelines
- **Minimum 95 percent test coverage** for impacted modules.
- Clear, reviewer-focused documentation.
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord for questions, reviews, and faster merges:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — if this issue and the maintainers helped you ship, we'd be grateful for a **5-star rating**. Clear questions in Discord and tidy, well-tested PRs are the fastest path to a merge and a reward.
++++++
---
type: Feature
title: "Document the schema-versioning and migration model for redeployments"
labels: type:docs, area:docs, stack:soroban, stack:rust, priority:low, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Document SchemaVersion, version(), and the migration workflow

### Description
[`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) distinguishes the compiled `version()` (returns 2) from the persisted `SchemaVersion` (`get_schema_version`, defaulting to 1) and exposes `migrate_v1_to_v2`, but the relationship and the operational workflow are documented only in scattered `///` comments. Operators have no guide for when and how to migrate. This issue captures the model.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add `docs/escrow/migrations.md` explaining: the difference between code `version()` and `SchemaVersion`; why v2 reads default sensibly for absent slots; how the `MigrationVersionMismatch (#11)` guard prevents double-runs.
- Provide a step-by-step migration runbook (deploy/redeploy → `migrate_v1_to_v2` → verify `get_schema_version`).
- Note the forward path for future `migrate_v2_to_v3`-style functions and the append-only versioning convention.
- Link the doc from [`README.md`](README.md).

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b docs/contracts-28-migration-docs`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — only `///` clarifications if needed.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — optionally a doc-aligned migration assertion.
  - **Add documentation:** add `docs/escrow/migrations.md`; link from [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) where gaps are found.
  - Validate security: migration cannot be run by non-admin (documented and cross-checked).
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: documenting the double-run guard behaviour accurately.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`docs: document schema-versioning and migration workflow`

### Guidelines
- **Minimum 95 percent test coverage** for impacted modules.
- Clear, reviewer-focused documentation.
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord for questions, reviews, and faster merges:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — if this issue and the maintainers helped you ship, we'd be grateful for a **5-star rating**. Clear questions in Discord and tidy, well-tested PRs are the fastest path to a merge and a reward.
++++++
---
type: Feature
title: "Extract shared admin-auth and pause-gate helpers to remove duplication"
labels: type:refactor, area:code-quality, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Refactor repeated admin-auth and pause checks into shared helpers

### Description
Nearly every admin entrypoint in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) repeats the same block: read `DataKey::Admin`, `unwrap_or_else` panic with `NotInitialized`, then `admin.require_auth()`. The pause check (`get(&DataKey::Paused).unwrap_or(false)` → panic `#4`) is likewise copy-pasted. This duplication is error-prone — a new entrypoint can easily forget a check. This issue centralises both into private helpers.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add private helpers, e.g. `fn require_admin(env: &Env) -> Address` and `fn ensure_not_paused(env: &Env)`, and route all existing entrypoints through them.
- Behaviour must be byte-for-byte identical: same error codes, same ordering of auth vs. pause checks, same panics.
- This is a pure refactor — no new public API, no semantic change; the existing test suite must pass unchanged.
- Add a short rationale comment explaining the helper pattern for future contributors.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b refactor/contracts-29-auth-pause-helpers`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — introduce helpers, replace duplicated blocks.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — existing tests must pass; add a regression test confirming a representative entrypoint still panics identically when paused/unauthorized.
  - **Add documentation:** note the helper convention in a module comment.
  - Include NatSpec-style doc comments (`///`) on the helpers.
  - Validate security: no check accidentally dropped during extraction.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: confirm `NotInitialized (#3)` and `ContractPaused (#4)` still fire from the helpers.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`refactor: extract shared admin-auth and pause-gate helpers`

### Guidelines
- **Minimum 95 percent test coverage** for impacted modules.
- Clear, reviewer-focused documentation.
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord for questions, reviews, and faster merges:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — if this issue and the maintainers helped you ship, we'd be grateful for a **5-star rating**. Clear questions in Discord and tidy, well-tested PRs are the fastest path to a merge and a reward.
++++++
---
type: Feature
title: "Reduce redundant storage reads in record_usage's validation chain"
labels: type:refactor, area:performance, stack:soroban, stack:rust, priority:low, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Refactor record_usage to minimise persistent storage reads

### Description
`record_usage` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) performs a long sequence of independent `env.storage().persistent().get(...)` calls — paused flag, max-per-call, min-per-call, require-registration, service-registered, service-disabled, allowlist-enabled, agent-allowed — before it even touches the usage counter. Each persistent read carries ledger cost. This issue tightens the validation chain to read only what is necessary and short-circuit cleanly, lowering the gas cost of the hottest entrypoint.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Reorder and gate reads so conditional reads (e.g. `ServiceRegistered`) only happen when their controlling flag (`RequireServiceRegistration`) is enabled — the current short-circuit already does some of this; verify and extend it.
- Avoid re-reading the same key twice within the call; cache locals where a value is used more than once.
- Behaviour must be identical: same validation order semantics where order is observable (which error fires first), same error codes.
- Document the read-count before/after in the PR to demonstrate the saving.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b refactor/contracts-30-record-usage-reads`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — tighten the read chain in `record_usage`.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — existing tests pass; add tests asserting error-precedence order is unchanged (e.g. paused beats zero-requests).
  - **Add documentation:** note the optimisation rationale in a code comment.
  - Include NatSpec-style doc comments (`///`) where helpers are introduced.
  - Validate security: no validation skipped, error precedence preserved.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: every gate still fires under its trigger condition; flags-off path reads the minimum.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`refactor: reduce redundant storage reads in record_usage validation`

### Guidelines
- **Minimum 95 percent test coverage** for impacted modules.
- Clear, reviewer-focused documentation.
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord for questions, reviews, and faster merges:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — if this issue and the maintainers helped you ship, we'd be grateful for a **5-star rating**. Clear questions in Discord and tidy, well-tested PRs are the fastest path to a merge and a reward.