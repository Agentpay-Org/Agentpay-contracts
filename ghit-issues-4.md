---
type: Feature
title: "Add a get_billing_summary read returning usage, price, and outstanding bill in one call"
labels: type:feature, area:queries, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Implement a combined billing-summary read for an agent-service pair

### Description
To render a single agent-service row, an off-chain dashboard today issues three separate host invocations against [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs): `get_usage` for the counter, `get_service_price` for the rate, and `compute_billing` for the product. These three reads can also race across ledgers, returning an internally inconsistent snapshot (a usage value from one ledger and a price from another). This issue adds a single read that returns all three values resolved from the same ledger state, so a caller gets a coherent billing snapshot in one round-trip.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add `get_billing_summary(env, agent, service_id) -> BillingSummary` where `BillingSummary { requests: u32, price_stroops: i128, billed: i128, last_settlement: Option<u64> }`, reusing `read_usage`, the `ServicePrice` read, the same `saturating_mul` as `compute_billing`, and the `LastSettlement` read so the batched and single paths cannot diverge.
- This is a pure read — no `require_auth`, no pause gate — consistent with the other getters; unknown pairs return zeroed fields and `None`.
- Add the `BillingSummary` struct as a `#[contracttype]` so client SDKs decode it; keep the change purely additive.
- Document that `billed == requests * price` with the documented saturation semantics.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feature/contracts-billing-summary`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — `BillingSummary` type and `get_billing_summary` reusing existing read helpers.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — summary matches the three individual getters; unknown pair returns zeros and `None`; values reflect post-settle drain.
  - **Add documentation:** document the combined read in [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: read-only, no auth bypass, consistent snapshot.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: free service (price 0), never-settled pair, post-settle zeroed usage, saturated billing.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`feat: add get_billing_summary combined read for an agent-service pair`

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
title: "Add a per-service price floor and ceiling enforced at set_service_price"
labels: type:feature, area:pricing, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Implement admin-configurable global price bounds for set_service_price

### Description
`set_service_price` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) accepts any non-negative `price_stroops` up to `i128::MAX`. There is no guardrail against a fat-fingered price that is orders of magnitude too high (which would over-bill every agent on the next settle) or an accidental near-zero price that gives the service away. This issue adds optional admin-configured global price bounds so a price outside the band is rejected before it lands on-chain.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add admin setters `set_price_bounds(min_stroops, max_stroops)` persisting `DataKey::MinServicePrice` / `DataKey::MaxServicePrice`, plus getters; default to no bounds (`0` and `i128::MAX`) so existing behaviour is unchanged.
- In `set_service_price`, after the existing negative-price and registration/disabled gates, reject a price below the floor or above the ceiling with a new `PriceOutOfBounds` error (next free code, append-only).
- Validate `min <= max` when the bounds are set, rejecting an inverted band with a typed error.
- Document the interaction with the zero-is-free semantics (decide whether a floor above 0 forbids free services and state it explicitly).

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feature/contracts-price-bounds`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — bound storage keys, setters/getters, the `set_service_price` band check, and the new error variant.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — price below floor / above ceiling rejected, in-band accepted, inverted band rejected, default unbounded.
  - **Add documentation:** document the price-bounds config in [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: bounds cannot be set by a non-admin; no price escapes the band when configured.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: exactly-at-floor, exactly-at-ceiling, zero price with a positive floor, bounds toggled then re-priced.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`feat: add per-service price floor and ceiling to set_service_price`

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
title: "Add a batched set_agent_allowed_batch to manage the allowlist in one transaction"
labels: type:feature, area:access-control, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Implement batched allowlist updates

### Description
`set_agent_allowed` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) flips the allowlist status for exactly one agent per admin transaction. Onboarding a fleet of agents — or revoking many during an incident — forces one transaction per address, each paying fixed Soroban call overhead and admin signing cost. This issue adds a batched setter so an admin can grant or revoke many agents atomically in a single call.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add `set_agent_allowed_batch(env, entries: Vec<(Address, bool)>)`: admin-gated via the existing pattern, honouring the pause gate, writing each `DataKey::AgentAllowed(addr)` through the shared `write_flag` helper.
- Bound the batch length with a documented constant and reject oversized batches with a typed error (append-only) to prevent unbounded-loop gas griefing.
- Define atomicity: the whole batch applies or none does (validate length before any write).
- Emit one summary event (e.g. `allow_bat(count)`) rather than per-entry spam, and document the choice.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feature/contracts-allowlist-batch`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — `set_agent_allowed_batch` reusing `write_flag` and the admin-auth helper.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — multi-agent grant/revoke, oversized batch rejected, empty batch no-op, `is_agent_allowed` reflects each entry.
  - **Add documentation:** document the batch setter and its bound in [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: only admin can call, bounded loop, no partial write on rejection.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: duplicate addresses in one batch, mix of grant/revoke, exactly-at-bound length, paused contract.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`feat: add batched set_agent_allowed_batch for fleet allowlist updates`

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
title: "Add a minimum-bill threshold so settle skips dust-sized invoices"
labels: type:feature, area:settlement, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Implement a configurable minimum settlement amount

### Description
`settle` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) drains and stamps any non-zero (or even zero) usage counter regardless of how small the resulting `billed` amount is. For a metered protocol this means a settlement transaction can cost more in fees than the dust amount it settles, and the on-chain `settled` event stream fills with negligible entries. This issue adds an admin-configured minimum-bill threshold below which `settle` refuses to drain, so settlements only happen when economically meaningful.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add admin `set_min_settlement_amount(stroops)` persisting `DataKey::MinSettlementAmount` (default `0` = always settle), plus a getter.
- In `settle`, after computing `billed`, if `billed < min_settlement_amount` reject the call with a new `BelowMinSettlement` error (append-only) so the counter is NOT drained and the dust accrues until it crosses the threshold.
- Preserve all existing gates (pause, admin/owner auth) and event semantics for settlements that do proceed.
- Document precisely that a zero-usage settle is rejected when a positive threshold is set, and that the counter is preserved on rejection.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feature/contracts-min-settlement`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — threshold key, setter/getter, the `settle` guard, and the new error variant.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — below-threshold rejected and counter preserved, at/above-threshold settles, default-zero always settles.
  - **Add documentation:** document the minimum-bill behaviour in [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: dust accrues rather than being lost; only admin sets the threshold.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: exactly-at-threshold, accrue-then-cross, free service with positive threshold, paused contract.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`feat: add minimum settlement amount to skip dust invoices`

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
title: "Track and expose a per-agent lifetime settled-amount counter"
labels: type:feature, area:usage-accounting, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Implement a lifetime settled-stroops accumulator per agent

### Description
[`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) tracks lifetime *request* counts (`TotalUsageByAgent`, `TotalRequestsAllTime`) but never accumulates the lifetime *value* settled. After `settle` drains a counter, the billed stroops are emitted in an event and then forgotten on-chain, so there is no queryable "how much has this agent ever paid" figure for SLA tiering, credit limits, or loyalty pricing without replaying the entire event log. This issue adds a lifetime settled-amount counter updated atomically inside `settle`.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add `DataKey::TotalSettledByAgent(Address)` (i128) and `DataKey::TotalSettledAllTime` (i128), incremented by `billed` inside `settle` using saturating arithmetic consistent with the existing counters.
- Add read entrypoints `get_total_settled_by_agent(agent)` and `get_total_settled_all_time()` defaulting to `0`.
- These counters are lifetime — `settle` must never decrement them (mirroring the documented invariant for the request counters).
- Keep the change additive: error codes untouched, the `settled` event payload unchanged.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feature/contracts-settled-amount-counter`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — new counter keys, increments inside `settle`, and the two getters.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — counters sum across multiple settles and agents, survive subsequent settles, saturate at `i128::MAX`.
  - **Add documentation:** document the settled-amount counters in [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: counters are monotonic and overflow-safe.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: zero-billed settle, multiple agents, saturation near `i128::MAX`.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`feat: track per-agent and protocol lifetime settled amounts`

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
title: "Add a decrement_usage entrypoint to correct over-reported usage before settlement"
labels: type:feature, area:usage-accounting, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Implement an admin-gated usage decrement for corrections

### Description
`record_usage` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) can only ever *increase* a `(agent, service_id)` counter, and the only way to lower it is `reset_usage`/`settle`, which wipe it entirely to zero. When a buggy metering client over-reports (say, double-counts a batch), the operator has no way to subtract the erroneous delta while keeping the legitimately accrued remainder. This issue adds a bounded, admin-gated decrement so a counter can be corrected downward without discarding the whole balance.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add `decrement_usage(env, agent, service_id, amount) -> u32`: admin-gated, pause-respecting, subtracting `amount` from the per-pair counter using `saturating_sub` (clamps at zero, never underflows) and returning the new total.
- Decide and document whether the lifetime `TotalUsageByAgent` / `TotalRequestsAllTime` counters are also adjusted (recommended: leave lifetime counters untouched so analytics retain the raw reported figure; document the choice clearly).
- Emit a distinct `usage_dec(agent, service_id, amount, new_total)` event so corrections are auditable and distinguishable from settlements.
- Reject `amount == 0` with the existing `RequestsMustBePositive` error to avoid no-op corrections in the audit trail.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feature/contracts-decrement-usage`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — `decrement_usage` with saturating subtraction and the correction event.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — decrement below current, decrement past zero clamps, lifetime counters unchanged, non-admin caller and paused contract rejected.
  - **Add documentation:** document the correction flow and lifetime-counter policy in [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: no underflow, only admin corrects, corrections are auditable.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: decrement equal to current (drains to zero), decrement on never-used pair (no-op clamp), zero amount rejected.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`feat: add admin-gated decrement_usage for over-report corrections`

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
title: "Allow an agent to authorize its own record_usage instead of an implicit caller"
labels: type:security, area:access-control, stack:soroban, stack:rust, priority:high, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Require agent authorization on record_usage

### Description
`record_usage` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) accepts an `agent: Address` parameter and writes usage against it, but **never calls `agent.require_auth()`** — any party can record usage on behalf of any agent address. While the metering loop is trusted today, this means a hostile caller can inflate a competitor agent's counters (and therefore its bill on the next settle) with no signature from the agent. This issue makes the recorded agent authorize the write, closing a usage-forgery vector.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add `agent.require_auth()` to `record_usage` (and any batched variant) so only the agent — or a metering operator the agent has authorized via Soroban's auth tree — can record against that agent.
- Consider and document an admin/operator override path so a trusted off-chain meter can still record on the agent's behalf when explicitly authorized; do not silently break the existing settlement-loop design — describe the migration.
- Preserve the full existing validation precedence; the auth check's position in the chain must be documented relative to the pause/zero/bounds gates.
- Keep error codes append-only.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b security/contracts-record-usage-auth`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — `agent.require_auth()` (and the documented override) in `record_usage`.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — using `env.mock_auths(&[...])` (scoped): authorized agent succeeds, unauthorized record fails, the operator override path works as documented.
  - **Add documentation:** document the new auth requirement and override in [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: usage cannot be forged against an unconsenting agent.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: missing signature, wrong signer, authorized operator, paused contract.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`security: require agent authorization on record_usage`

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
title: "Make transfer_service_ownership reject a no-op transfer to the current owner"
labels: type:security, area:service-registry, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Harden transfer_service_ownership against degenerate self-transfers

### Description
`transfer_service_ownership` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) re-writes `ServiceMetadata` and emits an `owner_chg(service_id, old_owner, new_owner)` event without checking whether `new_owner` differs from the existing owner. Transferring ownership to the address that already owns it is accepted, paying a storage write and emitting an `owner_chg` event whose `old_owner == new_owner` — which an indexer will mis-report as a meaningful handover. This mirrors the existing `InvalidAdminProposal` guard on `propose_admin_transfer` and should be applied here for consistency.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- In `transfer_service_ownership`, after loading the metadata and authorizing the caller, reject `new_owner == meta.owner` with a typed error (reuse the spirit of `InvalidAdminProposal`, or add a new append-only `InvalidOwnerTransfer` code — pick one and justify it).
- The no-op transfer must perform no storage write and emit no `owner_chg` event.
- Preserve all existing behaviour for genuine transfers and the missing-metadata / unauthorized-caller paths.
- Keep error codes append-only.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b security/contracts-owner-transfer-noop-guard`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — same-owner guard in `transfer_service_ownership`.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — transfer to current owner rejected with no event, genuine transfer still works, missing-metadata and wrong-caller paths unchanged.
  - **Add documentation:** note the no-op rejection in [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: no spurious `owner_chg` events, no wasted writes.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: admin transfers to current owner (rejected), owner transfers to self (rejected), valid handover, event-count assertion.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`security: reject no-op self-transfer in transfer_service_ownership`

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
title: "Honour the pause gate in set_agent_blocked, set_max_requests_per_window, and the rate-limit setters"
labels: type:security, area:access-control, stack:soroban, stack:rust, priority:high, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Close the pause-gate gaps on the newer admin setters

### Description
Most admin mutations in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) call `ensure_not_paused(&env)`, but several of the newer setters skip it: `set_agent_blocked`, `set_max_requests_per_window`, and `set_rate_window_seconds` authorize the admin and write directly **without consulting the `Paused` flag**. During an emergency pause an operator can therefore still mutate blocklist and rate-limit configuration, contradicting the contract's stated emergency-stop semantics. This issue brings those entrypoints in line with the rest of the admin surface.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add `ensure_not_paused(&env)` to `set_agent_blocked`, `set_max_requests_per_window`, and `set_rate_window_seconds` (and audit for any other setter that currently omits it), placed consistently with the existing setters (after the admin read + `require_auth`).
- Confirm `unpause` and all read entrypoints remain unaffected.
- This is a targeted consistency fix; do not change error codes or add new public API.
- Document which mutating entrypoints respect the pause gate so future setters follow the convention.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b security/contracts-pause-gate-newer-setters`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — `ensure_not_paused` in the identified setters.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — each fixed setter panics `#4` while paused, and works after unpause.
  - **Add documentation:** note the corrected pause coverage in [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: emergency stop now halts blocklist and rate-limit drift.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: pause then attempt each setter, unpause then succeed, non-admin caller still rejected.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`security: honour pause gate in blocklist and rate-limit setters`

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
title: "Validate min_requests_per_call against max_requests_per_call to prevent an unsatisfiable range"
labels: type:security, area:usage-accounting, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Reject a per-call floor that exceeds the per-call ceiling

### Description
`set_min_requests_per_call` and `set_max_requests_per_call` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) write their bounds independently with no cross-check. An admin can set `min > max`, which makes `record_usage` **unsatisfiable** — every call panics either `RequestsExceedsMaxPerCall (#8)` or `RequestsBelowMinPerCall (#9)` and no value can ever be recorded, silently bricking metering for every service until someone notices. This issue adds a consistency check so the two bounds can never be configured into a contradictory range.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- In `set_min_requests_per_call`, reject a `min` greater than the currently-stored `MaxRequestsPerCall` (defaulting to `u32::MAX`); in `set_max_requests_per_call`, reject a `max` less than the currently-stored `MinRequestsPerCall` (defaulting to `0`). Use a new `InvalidRequestBounds` error (append-only).
- Allow `min == max` (a fixed exact-count requirement) and document it.
- Preserve the existing default-unset behaviour and the `record_usage` precedence ordering.
- Document the invariant `min <= max` and the order operators should set the two values.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b security/contracts-request-bounds-consistency`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — cross-bound checks in both setters + the new error variant.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — setting min > max rejected, max < min rejected, min == max accepted, defaults still work.
  - **Add documentation:** document the `min <= max` invariant in [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: metering cannot be bricked by a contradictory range.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: set max then a too-high min, set min then a too-low max, equal bounds, unset one side.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`security: reject contradictory min/max per-call request bounds`

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
title: "Add a read entrypoint exposing the current rate-limit window state for an agent"
labels: type:feature, area:rate-limiting, stack:soroban, stack:rust, priority:low, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Implement get_rate_window to inspect an agent's live rate-limit state

### Description
The fixed-window rate limiter in `record_usage` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) stores per-agent `(window_start, count)` under `DataKey::RateWindow(Address)`, but there is **no read entrypoint** to inspect it. An agent or operator cannot ask "how many requests have I used in the current window and when does it reset?" without triggering a real `record_usage`, so clients cannot self-throttle or surface remaining quota in a UI. This issue exposes the window state read-only.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add `get_rate_window(env, agent) -> (u64, u32)` returning `(window_start, count)`, defaulting to `(0, 0)` when no window has opened, matching the `unwrap_or((0, 0))` used in `record_usage`.
- Optionally add a convenience `get_remaining_in_window(env, agent) -> u32` computed from `MaxRequestsPerWindow` and the current count, accounting for an expired window (returns the full cap when the window has rolled over); document the time-dependence on `env.ledger().timestamp()`.
- This is a pure read — no `require_auth`, no pause gate, no state mutation — consistent with the existing getters; do not roll the window forward as a side effect of reading.
- Keep the change additive.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feature/contracts-get-rate-window`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — `get_rate_window` (and optional `get_remaining_in_window`).
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — window state reflects recorded usage, reads do not mutate state, expired window reported correctly via `env.ledger().with_mut`.
  - **Add documentation:** document the rate-window reads in [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: read-only, no window advance on read.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: never-recorded agent, mid-window, just-expired window, limiter disabled.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`feat: add get_rate_window read for an agent's rate-limit state`

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
title: "Add an admin reset_rate_window entrypoint to clear an agent's throttle state"
labels: type:feature, area:rate-limiting, stack:soroban, stack:rust, priority:low, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Implement an admin override to reset a rate-limit window

### Description
Once an agent hits the per-window cap in `record_usage` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs), it must wait for the fixed window to roll over before it can record again — there is no operator override. If an agent was throttled in error (e.g. a misconfigured cap that was since raised, or a legitimate burst the operator wants to forgive), the admin has no way to clear the agent's `RateWindow` state. This issue adds an admin-gated reset so a throttle can be lifted immediately.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add `reset_rate_window(env, agent)`: admin-gated, pause-respecting, removing `DataKey::RateWindow(agent)` so the next `record_usage` opens a fresh window. Idempotent — resetting an agent with no window is a no-op.
- Emit a `rate_reset(agent)` event (additive) so the override is auditable.
- Document that this does not change the configured cap/window — it only clears the agent's accumulated count for the current window.
- Keep error codes append-only.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feature/contracts-reset-rate-window`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — `reset_rate_window` entrypoint + event.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — throttle an agent, reset, then record succeeds in the same ledger; reset on no-window is a no-op; non-admin caller and paused contract rejected.
  - **Add documentation:** document the operator override in [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: only admin can reset; an agent cannot self-clear its window.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: reset mid-window, reset with limiter disabled, reset then immediately re-throttle.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`feat: add admin reset_rate_window to clear an agent throttle`

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
title: "Emit a configuration-change event from every rate-limit and per-call bound setter"
labels: type:feature, area:events, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Add observability events for limit and bound configuration changes

### Description
The configuration setters in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — `set_max_requests_per_call`, `set_min_requests_per_call`, `set_max_requests_per_window`, `set_rate_window_seconds`, `set_require_service_registration`, `set_allowlist_enabled` — all mutate operational policy **silently**, emitting no event. Unlike `set_service_price` (which emits `price_set`) or `pause`, an indexer or security monitor has no on-chain signal when an operator tightens or loosens these limits. This issue adds a consistent config-change event to each, closing the observability gap for policy mutations.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Publish a topic-tagged event from each listed setter, e.g. `cfg_set(symbol_short!("max_call"), value)` style, or distinct topics per setter — pick a scheme with a single decodable schema and document it.
- Use `symbol_short!` topics (≤9 chars) and consistent data tuples; values that are `u32`/`u64`/`bool` should be encoded so one subscriber can decode all config events.
- Keep the change purely additive — do not alter the `price_set` or `paused` payloads, and do not reorder any existing logic.
- Document the full config-event catalogue (extending any existing events doc if present).

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feature/contracts-config-change-events`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — `env.events().publish(...)` in each config setter.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — assert the event topic/data via `env.events().all()` for each setter.
  - **Add documentation:** document the config events in [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: events expose no more than the state already does; topic lengths within Soroban symbol limits.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: setting a value to its current value still emits, boolean toggles, large numeric values.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`feat: emit config-change events from rate-limit and bound setters`

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
title: "Standardise the symbol_short event topic naming convention across all emitters"
labels: type:refactor, area:events, stack:soroban, stack:rust, priority:low, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Refactor event topics into named constants with a consistent scheme

### Description
Event topics in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) are scattered inline `symbol_short!` literals — `"usage"`, `"settled"`, `"price_set"`, `"price_rm"`, `"paused"`, `"svc_reg"`, `"owner_chg"`, `"meta_clr"` — with no central registry. The abbreviation style is inconsistent (`price_rm` vs `price_set`, `meta_clr` vs `owner_chg`), some are truncated to fit the 9-char limit in ad-hoc ways, and a typo in one literal would silently emit an undiscoverable topic. This issue centralises every topic into named constants so the scheme is consistent, auditable, and reused.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Define a module-level set of topic constants (e.g. `const TOPIC_USAGE: Symbol = symbol_short!("usage");`) for every emitted event and route all `env.events().publish` calls through them.
- This is a pure refactor — the emitted topics and payloads must be **byte-for-byte identical** to today so existing indexers are unaffected; only the source representation changes.
- Add a comment block documenting the topic scheme and the 9-char `symbol_short!` constraint so future events follow it.
- The existing test suite must pass unchanged; add a regression test asserting a representative event still publishes the exact same topic.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b refactor/contracts-event-topic-constants`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — topic constants + replace inline literals.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — existing tests pass; assert one event's topic is unchanged after the refactor.
  - **Add documentation:** note the topic-constant convention in a module comment.
  - Include NatSpec-style doc comments (`///`) on the constants block.
  - Validate security: no topic or payload changed; no silent typo introduced.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: every existing event topic matches its prior literal exactly.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`refactor: centralise event topics into named symbol constants`

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
title: "Extract the duplicated require_admin storage-read-and-auth block into a helper"
labels: type:refactor, area:code-quality, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Refactor the repeated admin read-and-require_auth idiom into require_admin

### Description
At least a dozen entrypoints in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — `set_service_price`, `remove_service_price`, `register_service`, `unregister_service`, `set_agent_allowed`, `set_agent_blocked`, `set_service_disabled`, `set_service_metadata`, `register_service_with_metadata`, the bounds/window setters, `pause`, `unpause`, `migrate_v1_to_v2`, and others — open with the identical block: `let admin = env.storage().persistent().get(&DataKey::Admin).unwrap_or_else(|| panic_with_error!(&env, EscrowError::NotInitialized)); admin.require_auth();`. This copy-paste is the single most-repeated pattern in the file and an easy place to introduce an ordering bug in a new entrypoint. This issue extracts it into one private helper.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add a private `fn require_admin(env: &Env) -> Address` that reads `DataKey::Admin`, panics `NotInitialized (#3)` if absent, calls `require_auth()`, and returns the admin address; route every admin entrypoint through it.
- Behaviour must be byte-for-byte identical: same `#3` error, same auth-then-pause ordering at each call site (the helper performs only the admin read + auth; the `ensure_not_paused` call remains where it currently is).
- This is a pure refactor — no public API or semantic change; the existing test suite must pass unchanged.
- Add a rationale comment so new entrypoints reuse the helper rather than re-inlining the block.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b refactor/contracts-require-admin-helper`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — `require_admin` helper, replace duplicated blocks.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — existing tests pass; add a regression test that a representative entrypoint still panics `#3` when uninitialized and rejects a wrong signer.
  - **Add documentation:** note the helper convention in a module comment.
  - Include NatSpec-style doc comments (`///`) on the helper.
  - Validate security: no auth check dropped, ordering preserved relative to pause checks.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: uninitialized contract panics `#3`, wrong signer rejected, correct admin succeeds.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`refactor: extract require_admin helper for the admin read-and-auth idiom`

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
title: "Replace the reused NotPendingAdmin error for unauthorized settle and ownership callers"
labels: type:security, area:access-control, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Add a dedicated Unauthorized error instead of overloading NotPendingAdmin

### Description
Both `settle` and `transfer_service_ownership` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) reject an unauthorized caller by panicking with `EscrowError::NotPendingAdmin (#6)` — an error whose name and doc comment are specifically about the two-step admin handover in `accept_admin_transfer`. A code comment even flags the reuse (`// reuse: unauthorized caller`). Client SDKs decoding `#6` cannot tell a failed admin acceptance from a generic unauthorized settle/ownership attempt, and operators reading logs are misled. This issue introduces a precise error while keeping codes append-only.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add a new `EscrowError::Unauthorized` variant (next free code, append-only) and use it for the unauthorized-caller branch in both `settle` and `transfer_service_ownership`.
- Do not renumber or change the meaning of `#6`; `accept_admin_transfer` keeps `NotPendingAdmin` for its genuine wrong-pending-caller case.
- Update the doc comments on `settle` and `transfer_service_ownership` (which currently document the `#6` reuse) to reference the new error.
- Keep all valid-path behaviour unchanged.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b security/contracts-unauthorized-error`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — new `Unauthorized` variant + swap in `settle` and `transfer_service_ownership`.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — unauthorized settle and transfer now panic the new code; `accept_admin_transfer` wrong-caller still panics `#6`.
  - **Add documentation:** note the error semantics in [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: no code reuse, stable SDK decoding for the admin-handover path.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: stranger settles (rejected new code), stranger transfers (rejected new code), wrong pending admin accepts (still `#6`).
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`security: add dedicated Unauthorized error for settle and ownership callers`

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
title: "Add tests for the per-agent fixed-window rate limiter across window rollovers"
labels: type:test, area:testing, stack:soroban, stack:rust, priority:high, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Test the rate-limit window math and RateLimitExceeded gate

### Description
The fixed-window rate limiter in `record_usage` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — `MaxRequestsPerWindow`, `WindowSeconds`, the `RateWindow(Address)` state, and the `RateLimitExceeded (#15)` error — has **no coverage** in [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs). Nothing asserts that the limiter is disabled by default, that a within-window over-cap call panics `#15`, or that the window correctly rolls forward once `now >= window_start + window_seconds`. This issue closes that gap by driving the ledger clock across window boundaries.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Cover: limiter disabled when either cap or window is `0` (default) allows unbounded calls; enabled + cumulative requests exceed cap within the window → panic `#15`; exactly-at-cap succeeds.
- Cover: after advancing the clock past the window via `env.ledger().with_mut`, the count resets and recording succeeds again; the window anchors at the first in-window call.
- Cover: an agent cannot reset its own window early (recording mid-window keeps `window_start` fixed).
- Use `#[should_panic(expected = "Error(Contract, #15)")]` matching the existing test conventions; test-only change unless a genuine bug surfaces.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b test/contracts-rate-limit-tests`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — no changes expected; only touch if a bug is found.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — the rate-limit scenarios above.
  - **Add documentation:** note the covered behaviour in the test module header comment.
  - Include NatSpec-style doc comments (`///`) on any test helpers.
  - Validate security: the throttle cannot be bypassed and cannot be reset early by the agent.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: cap reached then window rollover, single huge request exceeding cap, window length of one second, limiter half-configured (cap set, window zero).
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`test: cover fixed-window rate limiter and RateLimitExceeded gate`

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
title: "Add tests for the agent blocklist precedence over the allowlist in record_usage"
labels: type:test, area:testing, stack:soroban, stack:rust, priority:high, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Test the AgentBlocked gate and its precedence over the allowlist

### Description
The per-agent blocklist — `set_agent_blocked`, `is_agent_blocked`, and the `AgentBlocked (#17)` rejection in `record_usage` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — is uncovered by [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs). The contract documents a specific precedence: a blocked agent is rejected **even if also allow-listed**, and the block check runs before the allowlist check. Nothing proves this ordering, which is the entire security value of the feature. This issue locks the behaviour down with focused tests.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Cover: default (no block) allows the agent; blocked agent rejected with `#17`; unblock restores access; `is_agent_blocked` round-trips.
- Cover the precedence invariant: an agent that is both allow-listed (with the allowlist enabled) and blocked is still rejected with `#17` (block beats allow), and the block fires before the `AgentNotAllowed (#10)` path.
- Cover that a blocked agent is rejected regardless of the allowlist being enabled or disabled.
- Use `#[should_panic(expected = "Error(Contract, #17)")]`; test-only change unless a genuine bug surfaces.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b test/contracts-blocklist-precedence-tests`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — no changes expected; only touch if a bug is found.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — the blocklist and precedence scenarios above.
  - **Add documentation:** note the covered precedence in the test module header comment.
  - Include NatSpec-style doc comments (`///`) on any test helpers.
  - Validate security: the block cannot be circumvented via the allowlist.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: blocked + allow-listed, blocked + allowlist disabled, block then unblock then re-block, multiple agents with mixed status.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`test: cover agent blocklist precedence over the allowlist`

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
title: "Add tests for get_usage_batch ordering, defaults, and the BatchTooLarge bound"
labels: type:test, area:testing, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Test the batched usage read and its MAX_BATCH_READ guard

### Description
`get_usage_batch` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) returns one count per input `(agent, service_id)` pair, defaults unknown pairs to `0`, preserves input order, and panics with `BatchTooLarge (#16)` above `MAX_BATCH_READ` (100). None of this is covered by [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — there is no test that the order is preserved, that duplicate pairs yield the same value at each position, or that the bound actually fires. This issue adds the missing coverage for this read.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Cover: results are returned in the exact order of the input `pairs`; unknown pairs return `0`; a mix of known and unknown pairs maps positionally; duplicate pairs return the same value at each occurrence.
- Cover: a batch of exactly `MAX_BATCH_READ` succeeds; `MAX_BATCH_READ + 1` panics `#16`; an empty batch returns an empty `Vec`.
- Assert that `get_usage_batch` and `get_usage` agree for every shared pair (no drift between the single and batched read paths).
- Use `#[should_panic(expected = "Error(Contract, #16)")]`; test-only change unless a genuine bug surfaces.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b test/contracts-get-usage-batch-tests`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — no changes expected; only touch if a bug is found.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — the ordering, default, agreement, and bound scenarios above.
  - **Add documentation:** note the covered behaviour in the test module header comment.
  - Include NatSpec-style doc comments (`///`) on any test helpers.
  - Validate security: the read loop is bounded and cannot be forced past the cap.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: empty batch, single pair, exactly-at-bound, just-over-bound, all-unknown pairs.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`test: cover get_usage_batch ordering, defaults, and BatchTooLarge bound`

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
title: "Add tests for the owner-or-admin authorization branch in settle"
labels: type:test, area:testing, stack:soroban, stack:rust, priority:high, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Test the settle authorization matrix for admin, owner, and stranger callers

### Description
`settle` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) authorizes a `caller` that must be **either** the global admin **or** the `ServiceMetadata(service_id).owner`, with documented branches: admin always allowed, owner allowed for their own service, no-metadata + non-admin → `ServiceMetadataNotFound (#13)`, other address → the reused unauthorized error. The test suite relies on `mock_all_auths` and does not exercise these branches with scoped auth, so the entire owner-or-admin matrix is unverified. This issue adds focused authorization tests for `settle`.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Cover: admin settles any service; the registered owner settles their own service; the owner of service A cannot settle service B; a stranger is rejected.
- Cover: settling a service with no metadata as a non-admin panics `#13`; admin can still settle a metadata-less service.
- Use `env.mock_auths(&[...])` (scoped) so the negative cases genuinely prove an unauthorized caller fails, rather than blanket `mock_all_auths`.
- Assert the pause gate and the drain-to-zero + `LastSettlement` stamp still hold on the authorized path; test-only change unless a genuine bug surfaces.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b test/contracts-settle-authorization-tests`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — no changes expected; only touch if a bug is found.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — the authorization-matrix scenarios above with scoped auth.
  - **Add documentation:** note the covered auth matrix in the test module header comment.
  - Include NatSpec-style doc comments (`///`) on any test helpers.
  - Validate security: no caller outside admin/owner can settle a service.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: owner cross-service attempt, metadata-less service, paused contract, admin override.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`test: cover the owner-or-admin authorization matrix in settle`

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
title: "Add tests for the price-set events and the price-registration coupling in set_service_price"
labels: type:test, area:testing, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Test set_service_price events, gates, and the negative-price guard

### Description
`set_service_price` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) emits `price_set(service_id, price)`, rejects negative prices, and — when strict registration is enabled — rejects unregistered or disabled services with `ServiceNotRegistered (#7)` / `ServiceDisabled (#12)`. [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) does not assert the emitted event, the negative-price rejection, or the registration/disabled coupling. This issue adds the missing pricing-path coverage.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Cover: a successful `set_service_price` emits a `price_set` event with the expected `(service_id, price)` tuple via `env.events().all()`; the stored price round-trips through `get_service_price`.
- Cover: a negative price panics; a zero price is accepted (free service) and round-trips as `0`.
- Cover the coupling: with strict registration on, pricing an unregistered service panics `#7`; pricing a disabled service panics `#12`; with the flag off, pricing an unregistered service succeeds.
- Cover that `set_service_price` while paused panics `#4`; test-only change unless a genuine bug surfaces.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b test/contracts-set-price-tests`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — no changes expected; only touch if a bug is found.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — the event, negative-price, coupling, and pause scenarios above.
  - **Add documentation:** note the covered pricing behaviour in the test module header comment.
  - Include NatSpec-style doc comments (`///`) on any test helpers.
  - Validate security: prices cannot attach to disabled services; the event is emitted only after validation.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: re-pricing overwrites, price -1 rejected, zero price, strict-on vs strict-off.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`test: cover set_service_price events, gates, and negative-price guard`

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
title: "Add tests for remove_service_price and the zero-versus-removed price distinction"
labels: type:test, area:testing, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Test remove_service_price idempotency, events, and read-back semantics

### Description
`remove_service_price` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) frees the `ServicePrice(service_id)` slot, emits `price_rm(service_id)`, is admin-gated, pause-respecting, and idempotent — and the doc comment carefully distinguishes a removed price from `set_service_price(_, 0)` (both read back as `0`, but only removal reclaims the slot). [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) does not exercise any of this. This issue adds coverage for the removal entrypoint and the documented zero-vs-removed distinction.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Cover: set a price, remove it, then `get_service_price` reads `0` and `compute_billing` reads `0`; the `price_rm` event is emitted with the `service_id` via `env.events().all()`.
- Cover: removing a never-priced service is a no-op (still emits or not — assert the documented behaviour); removing while paused panics `#4`; a non-admin caller is rejected.
- Cover: remove then re-set restores a real price; both `set(_, 0)` and removal read back as `0` (documenting that the read cannot distinguish them).
- Test-only change unless a genuine bug surfaces.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b test/contracts-remove-price-tests`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — no changes expected; only touch if a bug is found.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — the removal, idempotency, event, and zero-vs-removed scenarios above.
  - **Add documentation:** note the covered semantics in the test module header comment.
  - Include NatSpec-style doc comments (`///`) on any test helpers.
  - Validate security: only admin can remove; no billing inconsistency after removal.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: remove unset price, remove then compute_billing, remove then re-price, paused removal.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`test: cover remove_service_price and the zero-vs-removed distinction`

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
title: "Add tests for register_service_with_metadata atomicity and the svc_reg event"
labels: type:test, area:testing, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Test the atomic registration-plus-metadata entrypoint

### Description
`register_service_with_metadata` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) sets `ServiceRegistered(service_id) = true` and writes `ServiceMetadata(service_id)` in one admin-gated call, emitting `svc_reg(service_id, owner)`. [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) does not verify that both slots are set by the single call, that the event carries the owner, or that the entrypoint honours the admin and pause gates. This issue adds the missing coverage for the combined entrypoint.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Cover: after one call, `is_service_registered` is `true` AND `get_service_metadata` returns the exact `description` + `owner`; the `svc_reg(service_id, owner)` event is emitted via `env.events().all()`.
- Cover: re-registering an existing id overwrites its metadata (idempotent overwrite); an empty description is accepted.
- Cover: a non-admin caller is rejected; calling while paused panics `#4`.
- Cover that the combined call is equivalent to the separate `register_service` + `set_service_metadata` sequence; test-only change unless a genuine bug surfaces.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b test/contracts-register-with-metadata-tests`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — no changes expected; only touch if a bug is found.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — the atomicity, event, idempotency, and gate scenarios above.
  - **Add documentation:** note the covered behaviour in the test module header comment.
  - Include NatSpec-style doc comments (`///`) on any test helpers.
  - Validate security: registration and metadata land together; only admin can call.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: overwrite existing service, empty description, paused contract, non-admin caller.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`test: cover register_service_with_metadata atomicity and svc_reg event`

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
title: "Add tests for transfer_service_ownership owner-or-admin auth and description preservation"
labels: type:test, area:testing, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Test the service-ownership handover entrypoint

### Description
`transfer_service_ownership` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) lets the current owner or the admin reassign `ServiceMetadata.owner` while preserving the description, emitting `owner_chg(service_id, old_owner, new_owner)`, panicking `ServiceMetadataNotFound (#13)` when no metadata exists, and rejecting any other caller. None of this is covered in [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs). This issue adds focused tests for the handover.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Cover: the current owner transfers to a new owner and the description is preserved; the admin transfers on the owner's behalf; the `owner_chg` event carries the correct old/new owners via `env.events().all()`.
- Cover: a third-party caller (neither owner nor admin) is rejected; transferring a service with no metadata panics `#13`.
- Cover that the pause gate fires (`#4`) when paused, and that after the transfer `get_service_metadata` reflects the new owner.
- Use `env.mock_auths(&[...])` (scoped) for the negative caller case; test-only change unless a genuine bug surfaces.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b test/contracts-ownership-transfer-tests`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — no changes expected; only touch if a bug is found.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — the handover, preservation, event, and gate scenarios above.
  - **Add documentation:** note the covered auth matrix in the test module header comment.
  - Include NatSpec-style doc comments (`///`) on any test helpers.
  - Validate security: only the current owner or admin can transfer; description is never lost.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: owner transfers, admin transfers, stranger rejected, missing metadata, paused contract.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`test: cover transfer_service_ownership auth and description preservation`

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
title: "Add tests for init idempotency, schema-version stamping, and the double-init guard"
labels: type:test, area:testing, stack:soroban, stack:rust, priority:high, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Test the init lifecycle and CURRENT_SCHEMA stamping

### Description
`init` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) stores the admin, stamps `DataKey::SchemaVersion` with `CURRENT_SCHEMA` (2), requires `admin.require_auth()`, and panics `AlreadyInitialized (#1)` on a second call. [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) does not assert that init stamps the current schema version, that a second init (even with the same admin) is rejected, or that `get_admin` reflects the stored admin. This issue locks the initialization lifecycle down.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Cover: after `init(admin)`, `get_admin` returns `Some(admin)` and `get_schema_version` returns `CURRENT_SCHEMA` (2) — proving a fresh v2 deploy is not mistaken for pre-migration v1.
- Cover: a second `init` (same or different admin) panics `#1`; the double-init guard cannot be bypassed.
- Cover: an admin-gated entrypoint before `init` panics `NotInitialized (#3)`.
- Cover the documented interaction: `migrate_v1_to_v2` on a freshly init-ed contract panics `MigrationVersionMismatch (#11)`; use `#[should_panic(expected = "...")]` matching the existing conventions; test-only change unless a genuine bug surfaces.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b test/contracts-init-lifecycle-tests`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — no changes expected; only touch if a bug is found.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — the init, schema-stamp, double-init, and migrate-rejection scenarios above.
  - **Add documentation:** note the covered lifecycle in the test module header comment.
  - Include NatSpec-style doc comments (`///`) on any test helpers.
  - Validate security: the contract cannot be re-initialized to seize admin.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: double init same admin, double init different admin, pre-init admin call, migrate on fresh deploy.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`test: cover init idempotency and CURRENT_SCHEMA stamping`

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
title: "Add tests for cancel_admin_transfer and re-proposal overwrite of a pending admin"
labels: type:test, area:testing, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Test the cancel and re-propose paths of the two-step admin handover

### Description
The two-step admin handover in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) supports cancelling a pending transfer (`cancel_admin_transfer`) and overwriting the pending entry by re-proposing (`propose_admin_transfer` re-write), with `get_pending_admin` exposing the state. [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) does not cover the cancel path, the re-propose-overwrites-pending behaviour, or the `InvalidAdminProposal (#14)` self-proposal guard. This issue fills those handover gaps.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Cover: propose then cancel clears `PendingAdmin` (`get_pending_admin` returns `None`); a subsequent `accept_admin_transfer` panics `NoPendingAdminTransfer (#5)`.
- Cover: re-proposing a different address overwrites the pending entry (`get_pending_admin` reflects the latest); accepting then uses the latest.
- Cover: proposing the current admin as the new admin panics `InvalidAdminProposal (#14)`.
- Cover: `cancel_admin_transfer` with nothing pending is a no-op; only the admin can cancel; test-only change unless a genuine bug surfaces.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b test/contracts-admin-cancel-repropose-tests`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — no changes expected; only touch if a bug is found.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — the cancel, re-propose, and self-proposal scenarios above.
  - **Add documentation:** note the covered handover behaviour in the test module header comment.
  - Include NatSpec-style doc comments (`///`) on any test helpers.
  - Validate security: the handover cannot be left in an inconsistent pending state.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: cancel with nothing pending, accept after cancel, re-propose twice, self-proposal rejected.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`test: cover cancel_admin_transfer and pending-admin re-proposal`

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
title: "Add tests for the usage event payload and the new-total return contract of record_usage"
labels: type:test, area:testing, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Test the record_usage event and UsageRecord return semantics

### Description
`record_usage` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) emits a `usage(agent, service_id, requests, total)` event and returns a `UsageRecord` carrying the **new total** (not the delta), per its doc comment. [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) does not assert the emitted event payload nor that the returned `requests` field equals the accumulated total rather than the per-call delta. This issue verifies both the event and the documented return contract.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Cover: after `record_usage(agent, svc, n)`, the returned `UsageRecord.requests` equals the accumulated total (e.g. two calls of 3 then 5 return 8, not 5); the event payload's `requests` field is the per-call delta while the `total` field is the running total.
- Cover: the `usage` event topic and tuple are emitted exactly once per call via `env.events().all()`.
- Cover: lifetime counters (`get_total_usage_by_agent`, `get_total_requests_all_time`) advance by the delta on each call.
- Test-only change unless a genuine bug surfaces.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b test/contracts-record-usage-event-tests`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — no changes expected; only touch if a bug is found.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — the return-value, event-payload, and lifetime-counter scenarios above.
  - **Add documentation:** note the covered contract in the test module header comment.
  - Include NatSpec-style doc comments (`///`) on any test helpers.
  - Validate security: the recorded total and emitted event agree, so off-chain loops cannot be misled.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: first call, repeated calls accumulating, multi-service isolation, single large delta.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`test: cover record_usage event payload and new-total return contract`

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
title: "Add a CHANGELOG and document the EscrowError code table through code 17"
labels: type:docs, area:docs, stack:soroban, stack:rust, priority:low, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Document the current error-code table and start a versioned changelog

### Description
[`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) now defines 17 `EscrowError` codes (through `AgentBlocked = 17`) added incrementally as features landed, but there is no single human-readable table mapping each code to its trigger and the entrypoints that raise it, and no changelog recording when each code or entrypoint was introduced. New integrators decoding a panic code must grep the source. This issue produces a precise error-code table and a `CHANGELOG.md` seeded with the current contract surface.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add `docs/escrow/errors.md` (or a README section) with a table: code number, variant name, trigger condition, and the entrypoint(s) that panic it — cross-checked against the `EscrowError` enum so it cannot drift, codes 1 through 17.
- Add `CHANGELOG.md` summarising the current v2 surface (entrypoints, events, error codes) as the baseline entry, and explain the append-only error-code and additive-event conventions so future changes are logged consistently.
- Note the overloaded codes that exist today (e.g. `RequestsMustBePositive` used for negative prices, `NotPendingAdmin` reused for unauthorized callers) so readers are not surprised.
- Link both documents from [`README.md`](README.md).

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b docs/contracts-error-table-changelog`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — only `///` clarifications if an error doc comment is found inaccurate.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — no new logic; optionally a smoke assertion if a code/doc mismatch is found.
  - **Add documentation:** add `docs/escrow/errors.md` and `CHANGELOG.md`; link from [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) where an error variant lacks one.
  - Validate security: the table accurately reflects every code so SDK authors decode correctly.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: every `EscrowError` variant appears exactly once in the table with the correct number.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`docs: add EscrowError code table and seed a CHANGELOG`

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
title: "Document the record_usage validation precedence chain as a reference table"
labels: type:docs, area:docs, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Document the fixed validation order of record_usage

### Description
`record_usage` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) enforces a fixed precedence — paused → zero-requests → max-per-call → min-per-call → registration → disabled → blocklist → allowlist → rate-limit — that an inline comment describes the contract relies on for stable client error ordering. But this precedence is documented only inside the source, and the comment does not yet reflect the blocklist and rate-limit checks added later. Integrators relying on which error fires first have no external reference. This issue produces an authoritative precedence document and reconciles the inline comment.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add `docs/escrow/validation-order.md` with a numbered table: each gate, the error it raises (with code), whether its read is conditional on a controlling flag, and the rationale for its position — covering the full current chain including `AgentBlocked (#17)` and `RateLimitExceeded (#15)`.
- Reconcile the inline precedence comment in `record_usage` (currently lists 1–7) with the actual implemented order so source and docs agree; note explicitly where the blocklist and rate-limit checks slot in.
- Explain why the ordering is part of the public contract (client SDKs and settlement loops depend on stable failure ordering).
- Link the doc from [`README.md`](README.md).

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b docs/contracts-validation-order`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — only `///`/inline comment fixes to reconcile the precedence comment with the implemented order.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — optionally a precedence assertion (e.g. paused beats zero-requests) if a discrepancy is found.
  - **Add documentation:** add `docs/escrow/validation-order.md`; link from [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) where the chain is referenced.
  - Validate security: documented order matches code so error handling cannot be mis-implemented.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: ensure every gate in the source appears in the table in the correct position.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`docs: document the record_usage validation precedence chain`

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
title: "Document the storage DataKey reference and persistent-entry layout"
labels: type:docs, area:docs, stack:soroban, stack:rust, priority:low, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Document every DataKey variant and its stored value type

### Description
The `DataKey` enum in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) now has more than 20 variants — config singletons (`Admin`, `Paused`, `SchemaVersion`, the bound/window settings), per-service slots (`ServicePrice`, `ServiceRegistered`, `ServiceDisabled`, `ServiceMetadata`), and per-agent or per-pair slots (`Usage`, `RateWindow`, `AgentAllowed`, `AgentBlocked`, `TotalUsageByAgent`, `LastSettlement`). Each is documented with a scattered inline comment, but there is no single map of the storage layout — what each key holds, its value type, its default when absent, and whether `settle` or migrations touch it. This issue produces that reference, which is essential for anyone reasoning about TTL, rent, or a future migration.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add `docs/escrow/storage.md` with a table: `DataKey` variant, key parameters, stored value type, default-when-absent (the `unwrap_or` value), which entrypoints write it, and whether it is lifetime/drained.
- Cross-check every variant against the source so the doc cannot drift; explain the persistent-storage model and why everything is `persistent()` (not `instance`/`temporary`).
- Note the per-pair vs per-agent vs singleton key cardinality so readers understand the rent footprint.
- Link the doc from [`README.md`](README.md).

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b docs/contracts-storage-reference`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — only `///` clarifications if a `DataKey` comment is found inaccurate.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — no new logic expected.
  - **Add documentation:** add `docs/escrow/storage.md`; link from [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) where a `DataKey` variant lacks one.
  - Validate security: the doc accurately states which keys are lifetime vs drained so accounting assumptions hold.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: every `DataKey` variant appears exactly once with its correct value type.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`docs: add storage DataKey reference and persistent-entry layout`

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
title: "Add a CI workflow running fmt, build, test, and clippy on every push"
labels: type:enhancement, area:ci, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Add a GitHub Actions CI pipeline for the escrow contract

### Description
Every issue in this repo asks contributors to run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`, but there is no automated CI to enforce that on pull requests — a PR can land with formatting drift, a warning, or a broken test if a reviewer misses it. The contract in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) is `#![no_std]` and built with a specific release profile, so a wasm-target build check is also valuable. This issue adds a CI workflow that runs the same gates the issues demand.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add `.github/workflows/ci.yml` running, on push and pull_request: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo build`, and `cargo test` for the workspace.
- Add a `wasm32-unknown-unknown` (or the Soroban target) build step that compiles the release profile defined in [`Cargo.toml`](Cargo.toml) so a non-building contract is caught.
- Pin the Rust toolchain (a `rust-toolchain.toml` or an action input) so CI is reproducible; cache the cargo registry/target to keep runs fast.
- Document the CI gates in [`README.md`](README.md) with a status badge.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b enhancement/contracts-ci-workflow`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — only fixes required to pass `clippy -D warnings` (e.g. trivial lint cleanups), kept minimal and behaviour-preserving.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — no new logic; ensure the suite passes under CI.
  - **Add documentation:** add the workflow, a `rust-toolchain.toml`, and a CI section + badge in [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) only where a clippy fix touches a public item.
  - Validate security: CI does not expose secrets; the pipeline only runs read/build/test steps.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test` locally to mirror CI.
- Cover edge cases: the workflow is valid YAML and the clippy gate passes with `-D warnings`.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`ci: add fmt, clippy, build, test, and wasm-build workflow`

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
title: "Add a shared test-harness module to remove setup boilerplate across tests"
labels: type:refactor, area:testing, stack:soroban, stack:rust, priority:low, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Refactor repeated test setup into reusable harness helpers

### Description
The tests in [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) repeat the same setup in nearly every case: create an `Env`, register the `Escrow` contract, build an `EscrowClient`, generate addresses, call `mock_all_auths`, and `init` with an admin. As the suite grows (and the many test-coverage issues in this campaign add more), this boilerplate is copy-pasted dozens of times, making tests verbose and easy to set up inconsistently. This issue extracts a small harness so new tests start from a single helper.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add private test helpers in [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs), e.g. `fn setup() -> (Env, EscrowClient, Address /*admin*/)` that registers the contract, mocks auths, and inits with a generated admin, plus small helpers for generating agents/services and advancing the ledger clock.
- This is a test-only refactor — no change to [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) and no change to what any test asserts; only the setup is centralised.
- Migrate the existing tests to the harness to prove it covers the real setup needs; keep each test's assertions identical.
- Document the harness conventions in the test module header so new tests reuse it.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b refactor/contracts-test-harness`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — no changes expected.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — add the `setup`/helper functions and migrate existing tests onto them; the full suite must pass unchanged.
  - **Add documentation:** note the harness convention in the test module header comment.
  - Include NatSpec-style doc comments (`///`) on the harness helpers.
  - Validate security: no assertion weakened; auth mocking remains explicit where a test needs scoped auth.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: a test needing scoped auth can opt out of the blanket-mock helper.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`refactor: add a shared test harness to remove setup boilerplate`

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
title: "Add an emergency admin-gated drain_usage_batch to zero many counters at once"
labels: type:feature, area:usage-accounting, stack:soroban, stack:rust, priority:low, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Implement a batched usage-zeroing entrypoint for incident response

### Description
There is no single-transaction way in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) to wipe usage across many `(agent, service_id)` pairs. After a metering bug that over-counted across a fleet, an operator must call `reset_usage`/`settle` once per pair to clean up — one transaction each, slow and expensive during an incident. This issue adds an admin-gated batched zeroing entrypoint distinct from settlement (no billing, no `LastSettlement` stamp) for fast incident cleanup.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add `drain_usage_batch(env, pairs: Vec<(Address, Symbol)>)`: admin-gated, pause-respecting, zeroing each `DataKey::Usage(agent, service_id)` without touching `LastSettlement`, lifetime counters, or emitting a billing event.
- Bound the batch length with a documented constant and reject oversized batches with a typed error (append-only) to keep the loop bounded.
- Emit one summary event (e.g. `drain_bat(count)`) for the audit trail, distinct from `settled` and any single-pair reset event.
- Document that this is a maintenance/incident tool, not a settlement path, and that lifetime analytics are intentionally preserved.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feature/contracts-drain-usage-batch`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — `drain_usage_batch` reusing the usage key and the admin-auth helper.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — multi-pair zeroing, lifetime counters and `LastSettlement` untouched, oversized batch rejected, non-admin and paused rejected.
  - **Add documentation:** document the incident-cleanup tool in [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: bounded loop, only admin, no billing side effects, analytics preserved.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: empty batch, never-used pairs, duplicate pairs, exactly-at-bound, paused contract.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`feat: add admin drain_usage_batch for fast incident cleanup`

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
title: "Add a get_contract_config read returning all global settings in one struct"
labels: type:feature, area:queries, stack:soroban, stack:rust, priority:low, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Implement a single read exposing the contract's global configuration

### Description
Inspecting the escrow's global configuration in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) today requires a fan-out of separate getters: `is_paused`, `is_allowlist_enabled`, `is_service_registration_required`, `get_max_requests_per_call`, `get_min_requests_per_call`, `get_max_requests_per_window`, `get_rate_window_seconds`, `get_schema_version`, and `get_admin`. A dashboard or health check must issue nine reads to render one config panel, and they can be inconsistent across ledgers. This issue adds a single read returning all global settings in one coherent struct.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add a `ContractConfig` `#[contracttype]` carrying `paused`, `allowlist_enabled`, `require_service_registration`, `max_requests_per_call`, `min_requests_per_call`, `max_requests_per_window`, `window_seconds`, `schema_version`, and `admin: Option<Address>`, and a `get_contract_config(env) -> ContractConfig` read that reuses the existing getters' default semantics.
- This is a pure read — no `require_auth`, no pause gate — consistent with the other getters; values must match the individual getters exactly (reuse them or the same `unwrap_or` defaults so they cannot drift).
- Keep the change purely additive; do not modify any existing getter.
- Document that the struct is a convenience snapshot and the per-field getters remain available.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feature/contracts-get-contract-config`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — `ContractConfig` type and `get_contract_config` reusing existing reads.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — every field matches its individual getter on a fresh contract and after several config changes.
  - **Add documentation:** document the config snapshot in [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: read-only, no auth bypass, fields cannot diverge from the per-field getters.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: defaults on a fresh contract, after toggling pause/allowlist/strict, after setting bounds and window.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`feat: add get_contract_config single-read configuration snapshot`

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
title: "Validate service_id against an empty symbol in registration and pricing entrypoints"
labels: type:security, area:service-registry, stack:soroban, stack:rust, priority:low, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Reject an empty service_id symbol across the service-scoped entrypoints

### Description
The service-scoped entrypoints in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — `register_service`, `register_service_with_metadata`, `set_service_price`, `set_service_metadata`, `set_service_disabled` — accept any `Symbol` as `service_id`, including the empty symbol. An empty `service_id` is almost certainly a client bug (an unset configuration field), yet it silently creates real registry/price/metadata entries under a meaningless key, which then accrue usage and rent and confuse dashboards. This issue rejects the empty symbol so a misconfiguration fails loudly instead of polluting state.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add a shared private check that rejects an empty `service_id` (length 0) with a new `InvalidServiceId` error (next free code, append-only), and apply it at the start of every service-mutating entrypoint listed above.
- Decide and document whether `record_usage` should also reject an empty `service_id` (recommended for consistency) and apply it there if so.
- Keep the check before any storage write so a bad id never lands on-chain; do not change behaviour for any non-empty id.
- Reuse a single helper so the rule cannot drift across entrypoints.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b security/contracts-reject-empty-service-id`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — shared empty-id guard + new error variant, applied to the service entrypoints.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — empty id rejected at each entrypoint, non-empty id still works, no partial write on rejection.
  - **Add documentation:** document the empty-id rejection in [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: state cannot be polluted with a meaningless key.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: empty id at register/price/metadata/disable, a one-char id accepted, record_usage with empty id (per the documented decision).
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`security: reject empty service_id in registration and pricing entrypoints`

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
title: "Add a get_services_status_batch read returning registered/disabled/price for many services"
labels: type:enhancement, area:queries, stack:soroban, stack:rust, priority:low, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Implement a batched per-service status read

### Description
To render a service catalogue, a dashboard must call `is_service_registered`, `is_service_disabled`, and `get_service_price` separately for every service — three reads per service, with no batched path in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs). For a catalogue of dozens of services this is many round-trips and risks an inconsistent cross-ledger snapshot. This issue adds a single bounded read returning the registration, disabled, and price status for a list of services.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add a `ServiceStatus` `#[contracttype]` carrying `service_id: Symbol`, `registered: bool`, `disabled: bool`, `price_stroops: i128`, and `get_services_status_batch(env, service_ids: Vec<Symbol>) -> Vec<ServiceStatus>` returning one entry per input id in order, reusing the existing `read_flag`/price-read defaults.
- Bound the input length with a documented constant (mirroring `MAX_BATCH_READ`) and reject oversized requests with a typed error (append-only) to keep the loop bounded.
- This is a pure read — no `require_auth`, no pause gate — consistent with the other getters; unknown services return `false`/`false`/`0`.
- Reuse the single-service read logic so the batched and single paths cannot drift.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b enhancement/contracts-services-status-batch`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — `ServiceStatus` type and `get_services_status_batch` reusing the per-service reads.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — order preserved, status matches the individual getters, unknown services default correctly, oversized batch rejected, empty batch returns empty vec.
  - **Add documentation:** document the batched status read and its bound in [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: bounded loop, read-only, no drift from single-service getters.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: duplicate ids, mix of registered/disabled/priced/unknown, exactly-at-bound length, empty batch.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`feat: add get_services_status_batch for catalogue rendering`

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
title: "Add a usage_threshold auto-flag event when a counter crosses a configurable level"
labels: type:feature, area:events, stack:soroban, stack:rust, priority:low, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Emit a threshold-crossed event so settlement can be triggered eagerly

### Description
The off-chain settlement loop must poll `get_usage` (or `compute_billing`) to discover when an agent-service counter has grown large enough to be worth settling — there is no on-chain push signal in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) when a counter crosses a meaningful level. This forces frequent polling. This issue adds a configurable per-call threshold so `record_usage` emits a distinct event the moment a counter crosses it, letting an event-driven settler react instead of poll.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add an admin-configurable `DataKey::UsageAlertThreshold` (default `0` = disabled) and a getter/setter.
- In `record_usage`, when the threshold is non-zero and the new per-pair total crosses it (was below, now at/above), emit a distinct `usage_hi(agent, service_id, total)` event in addition to the normal `usage` event — emit it only on the crossing edge, not on every subsequent call, so the event is not spammed.
- Keep the normal `usage` event and the return value unchanged; the change is additive and disabled by default.
- Document the edge-trigger semantics (fires once per window between settlements, re-arms after the counter drops below the threshold via settle/reset).

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feature/contracts-usage-threshold-event`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — threshold key, setter/getter, and the edge-triggered event in `record_usage`.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — event fires on the crossing call only, not on subsequent calls, re-arms after settle drains the counter, disabled by default emits nothing extra.
  - **Add documentation:** document the threshold alert in [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: the event leaks no more than the counter already exposes; edge-trigger cannot be spammed.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: exact-threshold crossing, single call jumping far past the threshold, threshold set to zero (disabled), crossing after a settle re-arm.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`feat: emit usage-threshold-crossed event for event-driven settlement`

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
title: "Add tests for compute_billing across free, priced, and saturated combinations"
labels: type:test, area:testing, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Test compute_billing's multiplication and zero/saturation behaviour

### Description
`compute_billing` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) returns `accumulated_requests * price_per_request` with `saturating_mul`, returns `0` when either side is zero, and is the read-only mirror of the math inside `settle`. [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) does not directly test `compute_billing` across its cases — zero usage, zero price (free service), a normal product, and the saturation edge. This issue adds focused coverage and verifies `compute_billing` agrees with the `billed` value `settle` would return for the same state.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Cover: zero usage → `0`; zero price (free service) → `0`; a normal `requests * price` product computed correctly; an unpriced and unused pair → `0`.
- Cover the saturation edge: a large `requests` × large `price` saturates at `i128::MAX` rather than overflowing.
- Cover that `compute_billing(agent, svc)` equals the value `settle` returns for the same pre-settle state (drive the state, read `compute_billing`, then `settle` and compare).
- Test-only change unless a genuine bug surfaces.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b test/contracts-compute-billing-tests`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — no changes expected; only touch if a bug is found.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — the zero, free, product, saturation, and settle-agreement scenarios above.
  - **Add documentation:** note the covered billing math in the test module header comment.
  - Include NatSpec-style doc comments (`///`) on any test helpers.
  - Validate security: billing math never overflows and the read mirrors settlement.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: zero usage, zero price, large product, saturated product, compute-vs-settle equality.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`test: cover compute_billing free, priced, and saturated cases`

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
title: "Add a renounce_admin path with an explicit two-step guard against accidental lockout"
labels: type:feature, area:access-control, stack:soroban, stack:rust, priority:low, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Implement a guarded admin renounce for finalising a contract

### Description
The admin in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) can only ever be rotated to another address via the two-step `propose_admin_transfer` / `accept_admin_transfer` handover — there is no way to **renounce** admin so the contract becomes immutable (no further price changes, registrations, or pauses). Some operators want to finalise a deployment to credibly signal it will not change. Renouncing is dangerous, so it must be deliberate. This issue adds a guarded renounce that cannot be triggered by a single fat-fingered call.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add a two-confirmation renounce: e.g. `propose_renounce()` sets a `DataKey::RenouncePending` flag, and `confirm_renounce()` (admin-authed again) removes `DataKey::Admin`, after which all admin-gated entrypoints panic `NotInitialized (#3)`. Document this is irreversible.
- Emit an `admin_rnc` event on confirmation so the finalisation is on-chain and auditable; clear any `PendingAdmin` on renounce.
- Provide `cancel_renounce()` to clear the pending flag, mirroring `cancel_admin_transfer`.
- Document the irreversibility loudly and the interaction with pause (recommend the contract be in a known state before renouncing).

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feature/contracts-renounce-admin`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — `propose_renounce`/`confirm_renounce`/`cancel_renounce`, the pending flag, and the event.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — renounce requires both steps, after renounce admin entrypoints panic `#3`, cancel clears the pending flag, non-admin cannot renounce.
  - **Add documentation:** document the irreversible renounce flow in [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: renounce cannot happen in one accidental call; reads still work after renounce.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: confirm without propose (rejected), cancel then confirm (rejected), reads work post-renounce, double renounce.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`feat: add guarded two-step admin renounce for finalising deployments`

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
title: "Add tests asserting reads and unpause remain callable while the contract is paused"
labels: type:test, area:testing, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Test that the pause gate blocks mutations but never reads or unpause

### Description
The pause gate in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) is meant to halt state-changing entrypoints while leaving the read getters and the `unpause` escape hatch fully usable — otherwise a paused contract could be permanently bricked or its state made unreadable during an incident. The test suite asserts some mutations are blocked but does not prove the complementary invariant: that every getter and `unpause` still work while paused. This issue locks down that "reads always work, recovery always works" guarantee.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Cover: with the contract paused, every read getter (`get_usage`, `get_usage_batch`, `get_service_price`, `compute_billing`, `get_service_metadata`, `is_*` flags, `get_admin`, `get_schema_version`, lifetime-counter reads) returns normally without panicking.
- Cover: `unpause` succeeds while paused and restores the ability to mutate; `record_usage` and `settle` panic `#4` while paused and succeed after unpause.
- Cover: pausing does not corrupt any previously-stored value (read a counter before pause, assert unchanged while paused).
- Test-only change unless a genuine bug surfaces.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b test/contracts-pause-reads-recovery-tests`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — no changes expected; only touch if a bug is found.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — the reads-work, unpause-works, and no-corruption scenarios above.
  - **Add documentation:** note the covered invariant in the test module header comment.
  - Include NatSpec-style doc comments (`///`) on any test helpers.
  - Validate security: a pause can always be lifted and never blinds the read surface.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: every getter while paused, record/settle blocked, unpause recovers, state intact across pause.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`test: assert reads and unpause stay callable while paused`

### Guidelines
- **Minimum 95 percent test coverage** for impacted modules.
- Clear, reviewer-focused documentation.
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord for questions, reviews, and faster merges:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — if this issue and the maintainers helped you ship, we'd be grateful for a **5-star rating**. Clear questions in Discord and tidy, well-tested PRs are the fastest path to a merge and a reward.
