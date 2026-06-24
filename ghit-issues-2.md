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