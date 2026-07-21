---
type: Feature
title: "Add register_service_with_metadata to register, describe, and assign an owner atomically"
labels: type:feature, area:service-registry, stack:soroban, stack:rust, priority:high, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Implement an atomic register-and-describe service entrypoint

### Description
Standing up a new service today takes three separate admin transactions against [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs): `register_service` (sets `DataKey::ServiceRegistered`), `set_service_metadata` (sets description + owner), and `set_service_price`. Between those calls the service exists in an inconsistent half-configured state — registered but ownerless, or described but unpriced — and an off-chain indexer that reacts to the registration sees no owner. This issue adds a single admin entrypoint that registers a service and writes its metadata atomically so a service is never observable in a partial state.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add `register_service_with_metadata(env, service_id, description, owner)` that sets `DataKey::ServiceRegistered(service_id) = true` and writes `DataKey::ServiceMetadata(service_id)` in one admin-gated call, honouring the existing `Admin` + `require_auth()` pattern.
- Reuse the same length/validation rules applied to `set_service_metadata` (do not duplicate logic — factor a shared private helper) so the description cap stays consistent.
- Emit a single event carrying `(service_id, owner)` so indexers learn of the registration and its owner together; keep it purely additive.
- Keep the standalone `register_service` and `set_service_metadata` entrypoints unchanged for backward compatibility.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feature/contracts-register-with-metadata`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — `register_service_with_metadata` plus a shared metadata-write helper.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — assert registration flag, metadata, and owner are all set in one call; `is_service_registered` and `get_service_metadata` round-trip; non-admin caller panics.
  - **Add documentation:** document the combined entrypoint in [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) on the new entrypoint, matching the existing style in `lib.rs`.
  - Validate security: only admin can register, no partial write on validation failure.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: re-registering an existing service (idempotent overwrite), empty description, non-admin caller, paused contract.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`feat: add register_service_with_metadata for atomic registration`

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
title: "Add remove_service_price to clear a configured price and revert a service to free"
labels: type:feature, area:pricing, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Implement a price-removal entrypoint for services

### Description
`set_service_price` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) can write or overwrite a price under `DataKey::ServicePrice(service_id)`, but there is **no way to remove the entry** — the only way to make a previously-priced service free again is to write `0`, which leaves a dangling stored key paying ledger rent and is semantically ambiguous (an explicit zero price vs. no price configured both read back as `0` from `get_service_price`). This issue adds an explicit removal entrypoint so operators can fully retire a price.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add `remove_service_price(env, service_id)`: admin-gated via the existing `Admin` + `require_auth()` pattern, calling `env.storage().persistent().remove(&DataKey::ServicePrice(service_id))`. Idempotent — removing an absent price is a no-op.
- Document that after removal `get_service_price` and `compute_billing` read back as the `0` default, identical to a never-priced service.
- Emit a `price_removed(service_id)` event (additive) so indexers can distinguish a removal from a `set` to zero.
- Honour the pause gate consistently with the other admin mutations.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feature/contracts-remove-service-price`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — `remove_service_price` entrypoint + event.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — set then remove returns `get_service_price == 0`; remove on never-priced service is a no-op; non-admin caller panics; event fires.
  - **Add documentation:** clarify the zero-vs-removed distinction in [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: only admin can remove, no fund/billing inconsistency after removal.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: remove then re-set, remove an unregistered service's price, compute_billing after removal.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`feat: add remove_service_price to clear a configured service price`

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
title: "Add a per-agent blocklist to deny specific agents independent of the allowlist"
labels: type:feature, area:access-control, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Implement an agent blocklist (deny list) in record_usage

### Description
Access control in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) is allowlist-only: `AllowlistEnabled` + `AgentAllowed(Address)` admit listed agents while it is on. There is no way to **deny a single abusive agent** while leaving the contract otherwise open — the operator must flip the global allowlist on and re-list every legitimate agent, which is operationally heavy during an incident. This issue adds a complementary deny list that blocks named agents regardless of the allowlist state.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add `DataKey::AgentBlocked(Address)` and admin `set_agent_blocked(agent, blocked)` / read `is_agent_blocked(agent)` using the existing admin-auth pattern.
- In `record_usage`, reject a blocked agent with a new `AgentBlocked` error (next free code, append-only) — the block must take precedence over the allowlist (a blocked agent is rejected even if also allow-listed).
- Document the precedence order (paused → zero → bounds → registration → disabled → **blocklist** → allowlist) precisely and decide where the new check slots in.
- Default to not-blocked (absent entry) so existing behaviour is unchanged when unused.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feature/contracts-agent-blocklist`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — blocklist key, setter/getter, and the `record_usage` deny check + new error variant.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — blocked agent rejected, block beats allowlist, unblock restores access, round-trip getters.
  - **Add documentation:** document the blocklist and precedence in [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: an agent cannot self-unblock, precedence is unambiguous.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: blocked + allow-listed, blocked while allowlist disabled, unblock then record, non-admin setter.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`feat: add per-agent blocklist with precedence over the allowlist`

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
title: "Add a batched get_usage_batch read for many agent-service pairs in one call"
labels: type:enhancement, area:queries, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Implement a batched usage read entrypoint

### Description
`get_usage` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) returns the counter for exactly one `(agent, service_id)` pair. A settlement loop or dashboard that needs the current usage for many pairs must issue one read invocation per pair, multiplying RPC round-trips and host invocations. This issue adds a single read that returns the counters for a caller-supplied list of pairs, with a bounded length to keep the call deterministic.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add `get_usage_batch(env, pairs: Vec<(Address, Symbol)>) -> Vec<u32>` returning one count per input pair, in input order, using the same `unwrap_or(0)` default as `get_usage`.
- Bound the input length with a documented `MAX_BATCH_READ` constant and reject oversized requests with a typed error so a caller cannot force an unbounded loop.
- This is a pure read — no `require_auth`, no state mutation, no pause gate (consistent with the existing getters).
- Reuse the single-pair read logic so the batch and single paths cannot drift.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b enhancement/contracts-get-usage-batch`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — `get_usage_batch` reusing the per-pair read.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — order preserved, unknown pairs return 0, oversized batch rejected, empty batch returns empty vec.
  - **Add documentation:** document the batch read and its bound in [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: bounded loop, read-only, no auth bypass.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: duplicate pairs in one request, mix of known/unknown pairs, exactly-at-bound length.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`feat: add batched get_usage_batch read for many pairs`

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
title: "Add transfer_service_ownership so a service owner can hand off their service"
labels: type:feature, area:service-registry, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Implement service-owner handover in metadata

### Description
The `owner` field of `ServiceMetadata` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) can only be changed by the admin re-writing the whole metadata via `set_service_metadata`. A service owner who wants to transfer their service to a new operator must ask the central admin to rewrite it, and the existing description is easy to clobber by accident. This issue adds a focused entrypoint that lets the current owner (or the admin) reassign ownership without touching the description.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add `transfer_service_ownership(env, service_id, new_owner)` that loads `DataKey::ServiceMetadata(service_id)`, replaces only `owner`, and re-stores it; the description is preserved.
- Authorize via the current `owner.require_auth()` OR the admin's `require_auth()`; reject with a typed error when no metadata/owner exists yet.
- Emit an `owner_chg(service_id, old_owner, new_owner)` event (additive) for indexers.
- Honour the pause gate consistently with other mutations and keep error codes append-only.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feature/contracts-transfer-service-ownership`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — `transfer_service_ownership` with owner-or-admin auth.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — owner transfers, admin transfers, description preserved, missing-metadata rejected, wrong caller rejected.
  - **Add documentation:** document the ownership-handover flow in [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: only current owner or admin can transfer; no description loss.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: transfer to self, transfer with no metadata, paused contract, third-party caller.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`feat: add transfer_service_ownership preserving description`

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
title: "Add clear_service_metadata to remove a service's description and owner"
labels: type:enhancement, area:service-registry, stack:soroban, stack:rust, priority:low, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Implement metadata removal for retired services

### Description
`set_service_metadata` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) can write or overwrite a `ServiceMetadata` entry, and `get_service_metadata` returns `Option<ServiceMetadata>`, but there is **no way to clear** an entry once written. When a service is retired with `unregister_service`, its description and owner linger under `DataKey::ServiceMetadata`, paying ledger rent and showing up as a phantom owner in dashboards. This issue adds an explicit metadata-removal entrypoint so a retirement can be clean.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add `clear_service_metadata(env, service_id)`: admin-gated via the existing pattern, calling `remove(&DataKey::ServiceMetadata(service_id))`. Idempotent — clearing an absent entry is a no-op.
- After clearing, `get_service_metadata` must read back as `None`, matching the never-set state.
- Emit a `meta_clear(service_id)` event (additive) so indexers can prune their view.
- Document the relationship with `unregister_service` (registration and metadata are independent slots; clearing one does not touch the other).

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b enhancement/contracts-clear-service-metadata`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — `clear_service_metadata` entrypoint + event.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — set then clear returns `None`; clear on never-set is a no-op; registration flag untouched; non-admin caller panics.
  - **Add documentation:** clarify metadata vs. registration lifecycle in [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: only admin can clear, no accidental registration removal.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: clear then re-set, clear after unregister, usage history preserved.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`feat: add clear_service_metadata for clean service retirement`

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
title: "Add tests for the settled event payload and the post-settle drain-to-zero invariant"
labels: type:test, area:testing, stack:soroban, stack:rust, priority:high, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Test the settle event payload and counter-drain behaviour

### Description
`settle` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) emits a `("settled",)` event carrying `(agent, service_id, requests, billed)` and stamps `LastSettlement` while draining the usage counter to zero. The existing test `test_settle_drains_usage_and_returns_billed` checks the return value and that usage is drained, but **no test asserts the emitted event payload** nor that `get_last_settlement` is stamped, and the `usage` event from `record_usage` is likewise unverified. This issue adds focused event-and-invariant coverage.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Cover: after `settle`, `env.events().all()` contains a `settled` event with the expected `(agent, service_id, requests, billed)` tuple; `get_usage` reads `0`; `get_last_settlement` returns `Some(timestamp)` matching the ledger clock set via `env.ledger().with_mut`.
- Cover: `record_usage` emits a `usage` event with `(agent, service_id, requests, total)`; re-recording after settle starts the per-pair counter from zero again.
- Assert the `compute_billing` value equals the `billed` returned by `settle` for the same pre-settle state.
- Test-only change unless a genuine bug surfaces.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b test/contracts-settle-event-invariants`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — no changes expected.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — the event and invariant scenarios above.
  - **Add documentation:** note the covered invariants in the test module header comment.
  - Include NatSpec-style doc comments (`///`) on any test helpers.
  - Validate security: settlement leaves no residual usage and produces an auditable event.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: settle of a zero-usage pair (zero-billed event), settle then record then settle again, multi-service event isolation.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`test: cover settle event payload and drain-to-zero invariant`

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
title: "Add tests for ServiceMetadata round-trip and registered/disabled slot independence"
labels: type:test, area:testing, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Test service metadata storage and slot independence

### Description
`set_service_metadata` / `get_service_metadata` and the independent `ServiceRegistered` and `ServiceDisabled` slots in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) are uncovered by [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs). The contract documents that these are **distinct slots** — a service can be registered but not disabled, disabled but still registered with metadata preserved — yet nothing proves that toggling one leaves the others intact. This issue adds coverage for the metadata round-trip and the slot-independence invariant.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Cover: `set_service_metadata` then `get_service_metadata` returns the exact `description` + `owner`; never-set service returns `None`.
- Cover: registering a service does not set its disabled flag; disabling a service preserves `ServiceRegistered` and `ServiceMetadata`; unregistering does not clear metadata or the disabled flag.
- Assert `is_service_registered` and `is_service_disabled` round-trips across the toggle matrix.
- Test-only change unless a genuine bug surfaces.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b test/contracts-metadata-slot-independence`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — no changes expected.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — the metadata and slot-independence scenarios above.
  - **Add documentation:** note covered invariants in the test module header.
  - Include NatSpec-style doc comments (`///`) on any test helpers.
  - Validate security: state slots cannot bleed into one another.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: overwrite metadata, disable an unregistered service, register-disable-unregister sequence.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`test: cover service metadata round-trip and slot independence`

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
title: "Add tests for pause/unpause event emission and idempotent toggling"
labels: type:test, area:testing, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Test the pause lifecycle events and idempotency

### Description
`pause` and `unpause` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) each publish a `("paused",)` event carrying `true`/`false` and are documented as idempotent. The existing tests `test_pause_admin_can_pause` and `test_unpause_admin_can_unpause` assert the `is_paused` state but **never check the emitted event** and never exercise the idempotent double-pause / double-unpause paths or a non-admin caller. This issue closes those gaps.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Cover: `pause` emits a `paused` event with `true`; `unpause` emits one with `false`, asserted via `env.events().all()`.
- Cover: pausing an already-paused contract and unpausing an already-unpaused contract are no-op writes that keep `is_paused` consistent (and document whether a duplicate event is still emitted).
- Cover: a non-admin caller to `pause`/`unpause` is rejected (use scoped auth, consistent with the convention used elsewhere).
- Test-only change unless a genuine bug surfaces.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b test/contracts-pause-events`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — no changes expected.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — the pause/unpause event and idempotency scenarios above.
  - **Add documentation:** note the covered behaviour in the test module header.
  - Include NatSpec-style doc comments (`///`) on any test helpers.
  - Validate security: the emergency-stop toggle behaves predictably under repeated calls.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: pause→pause→unpause sequence, unpause before any pause, event ordering.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`test: cover pause/unpause events and idempotent toggling`

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
title: "Reject same-address and self-targeted admin proposals in propose_admin_transfer"
labels: type:security, area:access-control, stack:soroban, stack:rust, priority:high, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Harden propose_admin_transfer against degenerate proposals

### Description
`propose_admin_transfer` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) stores any `new_admin` under `DataKey::PendingAdmin` without validating it. Proposing the **current admin as the new admin** is accepted, creating a pointless pending entry that an off-chain monitor will flag as an in-flight handover that never changes anything. There is also no guard that a stale `PendingAdmin` is cleared when a handover completes via a different path. This issue adds input validation so only meaningful proposals are stored.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- In `propose_admin_transfer`, reject a `new_admin` equal to the current admin with a new typed error (e.g. `InvalidAdminProposal`, next free code, append-only).
- Confirm `accept_admin_transfer` clears `PendingAdmin` (it does) and add a guard/test that no stale pending entry can survive a successful rotation.
- Decide and document whether re-proposing the same pending address is a no-op or an error; keep the two-step handover semantics intact.
- Keep all existing handover behaviour and error codes unchanged for the valid paths.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b security/contracts-admin-proposal-validation`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — validation in `propose_admin_transfer` + new error variant.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — proposing current admin panics with the new code; valid proposal still works; pending cleared after accept.
  - **Add documentation:** document the proposal validation rules in [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: no lockout path introduced, two-step handover still completable.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: propose self, propose then re-propose a different address, accept after a valid proposal.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`security: reject self-targeted proposals in propose_admin_transfer`

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
title: "Stamp SchemaVersion at init so fresh deploys are not mistaken for pre-migration v1"
labels: type:security, area:upgradeability, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Harden init to record the current schema version

### Description
`init` in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) stores only the admin and never writes `DataKey::SchemaVersion`. Because `get_schema_version` defaults to `1` when absent, a **freshly deployed v2 contract reports schema version 1** and is therefore eligible for `migrate_v1_to_v2` — a migration that is meaningless on a contract that was born at v2. This conflates "never migrated" with "freshly initialized at the current schema" and risks an unnecessary or confusing migration run. This issue makes `init` stamp the current schema version.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- In `init`, write `DataKey::SchemaVersion` with the current code schema (matching `version()`'s intent — define a `CURRENT_SCHEMA: u32` constant to avoid a magic number).
- Decide and document the interaction with `migrate_v1_to_v2`: a contract that was `init`-ed at v2 must **not** be migratable (it should panic `MigrationVersionMismatch`), while a genuinely upgraded-from-v1 deployment still migrates.
- Keep `get_schema_version`'s `unwrap_or(1)` default for the legacy pre-existing deployments that never stamped a version.
- Ensure the change does not break the existing double-init guard.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b security/contracts-init-stamps-schema-version`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — stamp `SchemaVersion` in `init`; add the `CURRENT_SCHEMA` constant.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — after `init`, `get_schema_version` returns the current version; `migrate_v1_to_v2` on a freshly init-ed contract panics `#11`.
  - **Add documentation:** clarify the init-vs-migrate distinction in [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) matching the existing style in `lib.rs`.
  - Validate security: no spurious migration on fresh deploys, legacy deployments unaffected.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: fresh init then migrate (rejected), legacy unset-version path still defaults to 1.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`security: stamp SchemaVersion at init to distinguish fresh v2 deploys`

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
title: "Write a build, test, and Soroban-CLI deployment guide for the escrow contract"
labels: type:docs, area:docs, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Document the build, test, and deploy workflow

### Description
[`README.md`](README.md) is brief and does not give a contributor an end-to-end path from a clone to a deployed, initialized contract on a Soroban network. There is no documented `stellar`/`soroban` CLI workflow for building the optimized wasm, running the test suite, deploying, and invoking `init` against the deployed id. New contributors must piece this together from the Soroban docs and the `Cargo.toml` profile by hand. This issue produces a concrete, copy-pasteable guide grounded in this repo's layout.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add `docs/escrow/build-deploy.md` covering: `cargo test` for the workspace, building the release wasm (referencing the `[profile.release]` settings in [`Cargo.toml`](Cargo.toml) — `opt-level = "z"`, `lto`, `panic = "abort"`), optional optimization, deploying with the Soroban/Stellar CLI to testnet, and invoking `init(admin)` on the deployed id.
- Note the workspace structure (`members = ["contracts/escrow"]`) and where the built artifact lands.
- Cross-reference the `version()` / `get_schema_version` check as a post-deploy sanity step.
- Keep commands accurate to the toolchain; mark any version-sensitive flags clearly. Link the guide from [`README.md`](README.md).

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b docs/contracts-build-deploy-guide`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — only `///` clarifications if an inline comment is found inaccurate.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — no new logic expected.
  - **Add documentation:** add `docs/escrow/build-deploy.md`; link from [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) where any are missing on referenced entrypoints.
  - Validate security: guide highlights that `init` must be run by the intended admin key.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: ensure documented commands match the actual workspace and profile settings.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`docs: add build, test, and Soroban-CLI deployment guide`

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
title: "Add a CONTRIBUTING guide documenting the append-only error-code and event conventions"
labels: type:docs, area:docs, stack:soroban, stack:rust, priority:low, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Document the contribution conventions for the escrow contract

### Description
The escrow contract in [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) follows several unwritten conventions that contributors must honour to avoid breaking client SDKs: error codes in `EscrowError` are **append-only** (the doc comment says so but the rule is not centralised), event topics use `symbol_short!` (≤9 chars), getters default sensibly via `unwrap_or`, and tests assert panics with `#[should_panic(expected = "Error(Contract, #N)")]`. A new contributor has no single document stating these rules. This issue captures them.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add `CONTRIBUTING.md` (or `docs/escrow/contributing.md`) documenting: append-only error codes (never renumber or reuse), the current code table, the `symbol_short!` topic constraint, the `unwrap_or` default convention for getters, the test panic-assertion style, and the `cargo fmt`/`cargo build`/`cargo test` gate.
- Explain the additive-only stance for events so existing payloads are not reshaped.
- Note the 95% coverage and 96-hour expectations referenced by the issue campaign.
- Link the guide from [`README.md`](README.md) and cross-reference any existing API/error docs.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b docs/contracts-contributing-guide`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — only `///` clarifications if a convention is mis-stated inline.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — no new logic expected.
  - **Add documentation:** add `CONTRIBUTING.md`; link from [`README.md`](README.md).
  - Include NatSpec-style doc comments (`///`) where a convention is referenced but undocumented in code.
  - Validate security: the guide reinforces the append-only error policy that keeps SDK decoding stable.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: ensure the documented error table exactly matches the current `EscrowError` variants and codes.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`docs: add CONTRIBUTING guide for error-code and event conventions`

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
title: "Extract typed boolean-flag storage accessors to remove unwrap_or(false) duplication"
labels: type:refactor, area:code-quality, stack:soroban, stack:rust, priority:low, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN
assignees: ''
---

## Refactor repeated persistent boolean-flag reads into shared accessors

### Description
[`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) repeats the same `env.storage().persistent().get(&DataKey::SomeFlag).unwrap_or(false)` pattern across many sites — the `Paused` check in both `record_usage` and `settle`, `RequireServiceRegistration`, `ServiceRegistered`, `ServiceDisabled`, `AllowlistEnabled`, `AgentAllowed`, and the `is_*` getters. This copy-paste makes it easy to typo a default or forget a check, and any new boolean flag re-introduces the same boilerplate. This issue centralises the read/write of boolean flags into small private helpers.

### Requirements and context
- **Repository scope:** `Agentpay-Org/Agentpay-contracts` only.
- Add private helpers, e.g. `fn read_flag(env: &Env, key: &DataKey) -> bool` (defaulting to `false`) and `fn write_flag(env: &Env, key: &DataKey, value: bool)`, and route the boolean reads/writes in `record_usage`, `settle`, the `set_*`/`is_*` flag entrypoints through them.
- Behaviour must be byte-for-byte identical: same defaults, same error codes, same ordering of checks; this is a pure refactor with no public API or semantic change.
- The existing test suite must pass unchanged; add a regression test confirming a representative flag still reads/writes/defaults identically.
- Add a short rationale comment so future flags use the helper.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b refactor/contracts-bool-flag-accessors`
- Implement changes
  - **Write code in:** [`contracts/escrow/src/lib.rs`](contracts/escrow/src/lib.rs) — introduce `read_flag`/`write_flag`, replace duplicated boolean reads/writes.
  - **Write comprehensive tests in:** [`contracts/escrow/src/test.rs`](contracts/escrow/src/test.rs) — existing tests pass; add a default-and-round-trip test for a representative flag.
  - **Add documentation:** note the helper convention in a module comment.
  - Include NatSpec-style doc comments (`///`) on the helpers.
  - Validate security: no flag default flipped, no check accidentally dropped during extraction.
- Test and commit

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
- Cover edge cases: unset flag defaults to false, set-true then set-false round-trip, paused gate still fires `#4`.
- Include the full `cargo test` output and a short **security notes** section in the PR description.

### Example commit message
`refactor: extract typed boolean-flag storage accessors`

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
