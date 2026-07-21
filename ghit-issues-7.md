---
type: Feature
title: "Add tests for open_dispute and resolve_dispute refund accounting"
labels: type:test, area:disputes, stack:soroban, stack:rust, priority:high, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Test add tests for open_dispute and resolve_dispute refund accounting

### Description
`open_dispute` and `resolve_dispute` mutate usage counters but have no dedicated coverage in `test.rs`. Tests should assert that a resolved dispute subtracts `refund_requests` from the stored `UsageRecord` and clears the open-dispute flag read by `has_open_dispute`.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Cover open -> resolve happy path and the `DisputeAlreadyOpen` and `NoOpenDispute` errors.
- Assert usage after refund equals the pre-dispute total minus `refund_requests`.
- Assert `has_open_dispute` returns false after resolution.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b test/contracts-add-tests-for-open-dispute-and`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`test(escrow): cover dispute open and resolve refund accounting`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Add tests for the RefundExceedsUsage guard in resolve_dispute"
labels: type:test, area:disputes, stack:soroban, stack:rust, priority:high, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Test add tests for the RefundExceedsUsage guard in resolve_dispute

### Description
`resolve_dispute` returns `EscrowError::RefundExceedsUsage` (code 22) when `refund_requests` exceeds the recorded usage, but this failure path is untested. Add cases exercising the boundary exactly at and one above the stored usage total.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Assert refunding exactly the full usage succeeds and drains the counter to zero.
- Assert refunding usage + 1 panics with `RefundExceedsUsage`.
- Assert a failed resolve leaves the dispute still open.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b test/contracts-add-tests-for-the-refundexceedsusage-guard`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`test(escrow): cover RefundExceedsUsage boundary in resolve_dispute`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Add tests for set_price_tiers validation and InvalidPriceTiers rejection"
labels: type:test, area:pricing, stack:soroban, stack:rust, priority:high, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Test add tests for set_price_tiers validation and InvalidPriceTiers rejection

### Description
`set_price_tiers` stores a `Vec<PriceTier>` and rejects malformed input with `EscrowError::InvalidPriceTiers` (code 18). The validation branches — empty vectors, non-monotonic thresholds, and negative prices — are not exercised in `test.rs`.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Cover empty, single-tier, and multi-tier vectors round-tripping through `get_price_tiers`.
- Assert out-of-order or duplicated tier thresholds panic with `InvalidPriceTiers`.
- Assert tiers are rejected when the contract is paused or the caller is not admin.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b test/contracts-add-tests-for-set-price-tiers`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`test(escrow): cover set_price_tiers validation and InvalidPriceTiers`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Add tests for tiered billing at exact tier boundary quantities"
labels: type:test, area:pricing, stack:soroban, stack:rust, priority:high, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Test add tests for tiered billing at exact tier boundary quantities

### Description
`compute_billing` resolves a `PriceTier` for the recorded usage, and off-by-one errors at tier edges would silently mis-bill agents. Add table-driven tests around each threshold in a multi-tier configuration.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Test usage at threshold - 1, exactly at threshold, and threshold + 1 for every tier.
- Assert usage above the highest tier uses the top tier price.
- Assert tiered billing takes precedence over the flat price set by `set_service_price`.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b test/contracts-add-tests-for-tiered-billing-at`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`test(escrow): cover tiered billing at tier boundaries`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Add tests for get_agent_usage_page offset and limit boundaries"
labels: type:test, area:queries, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Test add tests for get_agent_usage_page offset and limit boundaries

### Description
`get_agent_usage_page` paginates an agent's per-service usage but has no coverage for degenerate offsets and limits. Tests should pin the pagination contract so future refactors cannot silently drop or duplicate entries.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Cover zero limit, limit larger than the result set, and an offset past the end.
- Assert concatenating consecutive pages reproduces the full unpaginated result.
- Assert page ordering is stable across repeated calls.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b test/contracts-add-tests-for-get-agent-usage`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`test(escrow): cover get_agent_usage_page offset and limit edges`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Add tests for get_remaining_in_window after partial window consumption"
labels: type:test, area:rate-limit, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Test add tests for get_remaining_in_window after partial window consumption

### Description
`get_remaining_in_window` derives its value from `get_rate_window` and `get_max_requests_per_window`, but only the full-window rollover path is tested. Add coverage for partial consumption and for the value reported after the cap is reached.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Assert remaining decreases by the exact `requests` passed to `record_usage`.
- Assert remaining is zero once `RateLimitExceeded` would trigger.
- Assert remaining resets to the configured maximum after the window advances.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b test/contracts-add-tests-for-get-remaining-in`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`test(escrow): cover get_remaining_in_window partial consumption`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Add tests for settle_all result ordering and the SettleAllTooLarge bound"
labels: type:test, area:settlement, stack:soroban, stack:rust, priority:high, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Test add tests for settle_all result ordering and the SettleAllTooLarge bound

### Description
`settle_all` returns a `Vec<(Symbol, i128)>` and rejects oversized sweeps with `EscrowError::SettleAllTooLarge` (code 19). Neither the ordering guarantee nor the bound is asserted in `test.rs`.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Assert the returned pairs match per-service `compute_billing` values before the sweep.
- Assert exceeding the service-count bound panics with `SettleAllTooLarge`.
- Assert every swept service counter is drained to zero afterwards.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b test/contracts-add-tests-for-settle-all-result`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`test(escrow): cover settle_all ordering and SettleAllTooLarge`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Add tests for set_price_bounds rejecting PriceOutOfBounds and InvertedPriceBand"
labels: type:test, area:pricing, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Test add tests for set_price_bounds rejecting PriceOutOfBounds and InvertedPriceBand

### Description
`set_price_bounds` stores a min/max band enforced by `set_service_price` via `PriceOutOfBounds` (24) and `InvertedPriceBand` (25). These error paths lack tests despite guarding all pricing input.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Assert `set_price_bounds(min, max)` with min > max panics with `InvertedPriceBand`.
- Assert prices below `get_min_service_price` or above `get_max_service_price` are rejected.
- Assert tightening the band does not retroactively invalidate stored prices.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b test/contracts-add-tests-for-set-price-bounds`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`test(escrow): cover price bound validation errors`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Add tests for remove_price_tiers falling back to the flat service price"
labels: type:test, area:pricing, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Test add tests for remove_price_tiers falling back to the flat service price

### Description
`remove_price_tiers` clears the tier vector, after which `compute_billing` must fall back to the flat price from `set_service_price`. This transition is untested and is an easy place to leave stale tiers behind.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Assert `get_price_tiers` returns `None` after removal.
- Assert billing switches from tiered to flat pricing for identical usage.
- Assert removing tiers on a service that never had them is idempotent.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b test/contracts-add-tests-for-remove-price-tiers`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`test(escrow): cover remove_price_tiers flat-price fallback`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Add tests for unregister_service and its effect on previously recorded usage"
labels: type:test, area:registry, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Test add tests for unregister_service and its effect on previously recorded usage

### Description
`unregister_service` removes a service from the registry while historical `UsageRecord` entries remain. Tests should document whether outstanding usage stays settleable after unregistration, which is currently unspecified behaviour.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Assert `is_service_registered` flips to false and `record_usage` then fails with `ServiceNotRegistered` when registration is required.
- Assert previously recorded usage is still readable via `get_usage`.
- Assert `settle` behaviour on an unregistered service is deterministic and documented.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b test/contracts-add-tests-for-unregister-service-and`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`test(escrow): cover unregister_service and residual usage`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Add tests for get_agent_services membership after reset and settlement"
labels: type:test, area:queries, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Test add tests for get_agent_services membership after reset and settlement

### Description
`get_agent_services` tracks which services an agent has touched, but nothing asserts whether entries are removed after a counter is drained by `settle`. Add tests locking in the intended membership semantics.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Assert a service appears exactly once after repeated `record_usage` calls.
- Assert membership after `settle` drains the counter to zero.
- Assert the list is empty for an agent with no usage.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b test/contracts-add-tests-for-get-agent-services`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`test(escrow): cover get_agent_services membership semantics`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Add tests for decrement_usage saturation below zero and its event payload"
labels: type:test, area:usage, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Test add tests for decrement_usage saturation below zero and its event payload

### Description
`decrement_usage` returns the new `u32` total and must not underflow when `amount` exceeds the stored usage. Add coverage for saturation and for the event emitted on adjustment.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Assert decrementing more than the stored usage saturates at zero rather than wrapping.
- Assert the returned value equals the value later read by `get_usage`.
- Assert the emitted event topic and payload match the documented convention.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b test/contracts-add-tests-for-decrement-usage-saturation`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`test(escrow): cover decrement_usage saturation and event`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Bound the tier vector length in set_price_tiers to cap settlement gas"
labels: type:security, area:disputes, stack:soroban, stack:rust, priority:high, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Harden bound the tier vector length in set_price_tiers to cap settlement gas

### Description
`set_price_tiers` accepts an unbounded `Vec<PriceTier>`, and `compute_billing` walks that vector on every billing read. A large tier list makes `settle` and `settle_all` arbitrarily expensive or unexecutable.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Introduce a `MAX_PRICE_TIERS` constant and reject longer vectors with `InvalidPriceTiers`.
- Document the bound alongside the existing `BatchTooLarge` limits.
- Add tests at the limit and one entry above it.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b security/contracts-bound-the-tier-vector-length-in`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`security(escrow): bound price tier vector length`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Restrict resolve_dispute authorization to the service owner or admin"
labels: type:security, area:disputes, stack:soroban, stack:rust, priority:high, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Harden restrict resolve_dispute authorization to the service owner or admin

### Description
`resolve_dispute` adjusts usage downward and therefore reduces the amount an agent owes. Its authorization must be at least as strict as `settle`, which accepts the service owner from `ServiceMetadata` or the admin.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Require `require_auth` on a caller matched against `ServiceMetadata.owner` or the stored admin.
- Return `EscrowError::Unauthorized` (code 26) for any other caller.
- Add tests for owner, admin, and unauthorized third-party callers.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b security/contracts-restrict-resolve-dispute-authorization-to-the`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`security(escrow): restrict resolve_dispute to owner or admin`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Block settle and settle_all while a dispute is open on the pair"
labels: type:security, area:settlement, stack:soroban, stack:rust, priority:high, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Harden block settle and settle_all while a dispute is open on the pair

### Description
`settle` currently drains a counter regardless of `has_open_dispute`, so a service owner can settle a contested invoice before `resolve_dispute` runs. Settlement must be gated on the dispute flag.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Reject `settle` with a dedicated error when `has_open_dispute` is true for the agent-service pair.
- Make `settle_all` skip or reject disputed services consistently and document the choice.
- Add tests covering settle attempts before and after `resolve_dispute`.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b security/contracts-block-settle-and-settle-all-while`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`security(escrow): block settlement of disputed invoices`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Cap get_agent_services growth to prevent an unbounded-read denial of service"
labels: type:security, area:queries, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Harden cap get_agent_services growth to prevent an unbounded-read denial of service

### Description
The service list backing `get_agent_services` grows once per distinct service an agent touches and is read wholesale by `settle_all`. Without a cap, an agent interacting with many services can make its own settlement path unexecutable.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Enforce a maximum tracked-service count in `record_usage` with a clear error.
- Ensure `settle_all` remains executable at the cap within ledger limits.
- Add tests recording usage up to and past the cap.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b security/contracts-cap-get-agent-services-growth-to`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`security(escrow): cap tracked services per agent`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Enforce state mutation before event emission ordering across settle paths"
labels: type:security, area:events, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Harden enforce state mutation before event emission ordering across settle paths

### Description
`settle` and `settle_all` both write storage and emit events; any path that emits before the counter is drained can publish a bill that a later failure rolls back. Audit and normalise the ordering so events only describe committed state.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Review every emitter in `lib.rs` and move emission after the final storage write.
- Add a short convention note to CONTRIBUTING covering emit-after-commit.
- Add tests asserting no event is observed when a settle path panics.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b security/contracts-enforce-state-mutation-before-event-emission`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`security(escrow): emit events only after state commits`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Centralise batch-length limits for get_usage_batch, settle_all, and price tiers"
labels: type:security, area:validation, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Harden centralise batch-length limits for get_usage_batch, settle_all, and price tiers

### Description
Batch bounds are enforced separately by `get_usage_batch` (`BatchTooLarge`, 16) and `settle_all` (`SettleAllTooLarge`, 19), with no bound at all on `set_price_tiers`. Divergent limits are easy to skew during future edits.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Introduce named constants for each bound in one place near the top of `lib.rs`.
- Add a shared `require_batch_len` helper used by all three entrypoints.
- Add tests asserting each entrypoint rejects exactly one element over its bound.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b security/contracts-centralise-batch-length-limits-for-get`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`refactor(escrow): centralise batch length limits`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Add a dispute window so usage cannot be contested after settlement finality"
labels: type:security, area:disputes, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Harden add a dispute window so usage cannot be contested after settlement finality

### Description
`open_dispute` can be called at any time, including long after `get_last_settlement` records a settlement. A configurable dispute window gives settlements finality and bounds how far back refunds can reach.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Add an admin-settable `dispute_window_seconds` compared against `get_last_settlement`.
- Reject `open_dispute` outside the window with a dedicated error code.
- Add tests advancing the ledger timestamp across the window boundary.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b security/contracts-add-a-dispute-window-so-usage`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`feat(escrow): add a bounded dispute window`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Add a protocol fee in basis points deducted at settle"
labels: type:feature, area:fees, stack:soroban, stack:rust, priority:high, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Add add a protocol fee in basis points deducted at settle

### Description
`settle` currently returns the full billed amount to the service owner with no protocol take. Adding an admin-configurable basis-point fee lets the network capture revenue without changing the `compute_billing` contract.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Add `set_protocol_fee_bps` / `get_protocol_fee_bps` with a hard cap well below 10000.
- Split the settled amount in `settle` and `settle_all` and emit both components.
- Use saturating arithmetic so fee math cannot overflow `i128`.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feat/contracts-add-a-protocol-fee-in-basis`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`feat(escrow): add basis-point protocol fee at settlement`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Add a treasury address receiving accrued protocol fees"
labels: type:feature, area:fees, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Add add a treasury address receiving accrued protocol fees

### Description
A protocol fee is only useful with a destination. Store an admin-settable treasury `Address` and accrue the fee portion of every `settle` against it so it can be read and later withdrawn.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Add `set_treasury` / `get_treasury` guarded by the same admin check as `set_service_price`.
- Track cumulative accrued fees alongside `get_total_settled_all_time`.
- Reject settlement when a non-zero fee is configured but no treasury is set.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feat/contracts-add-a-treasury-address-receiving-accrued`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`feat(escrow): add a protocol fee treasury address`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Add prepaid agent credit balances drawn down during settle"
labels: type:feature, area:credits, stack:soroban, stack:rust, priority:high, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Add add prepaid agent credit balances drawn down during settle

### Description
AgentPay meters usage post-paid today: `record_usage` accrues and `settle` bills. Prepaid credits let an agent fund a balance up front that `settle` draws down, giving services a solvency guarantee before work is performed.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Add `credit_agent` / `get_agent_credit` and deduct from the balance inside `settle`.
- Reject `record_usage` when credits are enabled and the balance cannot cover the minimum call.
- Emit a credit-debited event and add tests for partial and insufficient balances.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feat/contracts-add-prepaid-agent-credit-balances-drawn`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`feat(escrow): add prepaid agent credit balances`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Add a per-agent spending cap enforced in compute_billing"
labels: type:feature, area:limits, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Add add a per-agent spending cap enforced in compute_billing

### Description
Rate limiting bounds request counts but not spend, so a single high-priced service can produce an unbounded invoice. A per-agent cap checked during `compute_billing` gives operators a monetary safety limit.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Add `set_agent_spend_cap` / `get_agent_spend_cap` with zero meaning unlimited.
- Reject `record_usage` when the projected bill would exceed the cap.
- Add tests at, just below, and just above the configured cap.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feat/contracts-add-a-per-agent-spending-cap`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`feat(escrow): add per-agent spending caps`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Add per-service usage quotas that auto-disable a service on breach"
labels: type:feature, area:limits, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Add add per-service usage quotas that auto-disable a service on breach

### Description
`set_service_disabled` is purely manual today. A per-service quota that flips the disabled flag automatically when cumulative usage crosses a threshold protects capacity-limited services without operator intervention.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Add `set_service_quota` / `get_service_quota` and check it inside `record_usage`.
- Set the existing disabled flag and emit an event when the quota is breached.
- Allow an admin to clear the quota and re-enable the service.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feat/contracts-add-per-service-usage-quotas-that`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`feat(escrow): add per-service usage quotas`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Add a get_global_stats read returning all aggregate counters"
labels: type:feature, area:queries, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Add add a get_global_stats read returning all aggregate counters

### Description
`get_total_requests_all_time`, `get_total_settled_all_time`, and the per-agent counters each require a separate contract call. A single aggregate read cuts round trips for dashboards and indexers.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Return a struct combining all-time requests, all-time settled, and registered service count.
- Keep the struct additive so future counters do not break existing consumers.
- Add tests asserting the aggregate matches the individual reads.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feat/contracts-add-a-get-global-stats-read`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`feat(escrow): add aggregate get_global_stats read`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Add a settle_due sweep for invoices past a configurable settlement interval"
labels: type:feature, area:settlement, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Add add a settle_due sweep for invoices past a configurable settlement interval

### Description
`get_last_settlement` already records when a pair was last settled, but nothing acts on it. A `settle_due` entrypoint that only settles pairs older than a configured interval enables periodic keeper-driven billing.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Add `set_settlement_interval` and a `settle_due(caller, agent)` sweep filtering on `get_last_settlement`.
- Skip pairs with zero billing and pairs with an open dispute.
- Add tests advancing the ledger timestamp across the interval boundary.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feat/contracts-add-a-settle-due-sweep-for`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`feat(escrow): add settle_due periodic settlement sweep`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Add per-service settlement token selection for multi-asset billing"
labels: type:feature, area:tokens, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Add add per-service settlement token selection for multi-asset billing

### Description
Prices are stored in stroops with an implicit single asset. Storing a settlement token `Address` per service in `ServiceMetadata` lets services bill in the Stellar Asset Contract of their choice.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Add `set_service_token` / `get_service_token` with a contract-wide default.
- Include the resolved token in the settled event payload.
- Add tests for default fallback and per-service overrides.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feat/contracts-add-per-service-settlement-token-selection`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`feat(escrow): add per-service settlement token`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Add a cancel_dispute entrypoint for the agent that opened it"
labels: type:feature, area:disputes, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Add add a cancel_dispute entrypoint for the agent that opened it

### Description
Once `open_dispute` succeeds, only `resolve_dispute` can clear the flag, so an agent that opens a dispute in error cannot withdraw it and settlement stays blocked. Add a self-service cancellation path.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Add `cancel_dispute(agent, service_id)` requiring the disputing agent's `require_auth`.
- Return `NoOpenDispute` (code 21) when no dispute exists.
- Leave usage counters untouched and emit a cancellation event.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feat/contracts-add-a-cancel-dispute-entrypoint-for`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`feat(escrow): add cancel_dispute entrypoint`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Add allowlist entries with an expiry timestamp"
labels: type:feature, area:allowlist, stack:soroban, stack:rust, priority:low, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Add add allowlist entries with an expiry timestamp

### Description
`set_agent_allowed` grants access permanently until explicitly revoked. Time-bounded entries let operators issue trial or contract-term access that lapses without a follow-up transaction.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Extend the allowlist entry to store an expiry ledger timestamp, zero meaning never.
- Have `is_agent_allowed` and `record_usage` treat expired entries as not allowed.
- Add tests advancing the ledger timestamp past expiry.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feat/contracts-add-allowlist-entries-with-an-expiry`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`feat(escrow): add expiring allowlist entries`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Add a flat-rate subscription billing mode per service"
labels: type:feature, area:pricing, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Add add a flat-rate subscription billing mode per service

### Description
Pricing is strictly per-request via `set_service_price` and `set_price_tiers`. A subscription mode charges a fixed periodic amount regardless of request count, which suits always-on agent services.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Add a per-service subscription price and period alongside the existing pricing modes.
- Make `compute_billing` return the subscription amount when the mode is active.
- Document mode precedence between subscription, tiered, and flat pricing.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feat/contracts-add-a-flat-rate-subscription-billing`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`feat(escrow): add flat-rate subscription billing mode`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Add a list_open_disputes read for an agent"
labels: type:feature, area:disputes, stack:soroban, stack:rust, priority:low, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Add add a list_open_disputes read for an agent

### Description
`has_open_dispute` answers only for a single agent-service pair, so a client must probe every service to find contested invoices. A list read makes dispute state discoverable in one call.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Return the disputed `Symbol` service ids for an agent, bounded by the shared batch limit.
- Reuse the tracked service list backing `get_agent_services` as the iteration source.
- Add tests for zero, one, and many open disputes.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feat/contracts-add-a-list-open-disputes-read`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`feat(escrow): add list_open_disputes read`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Add an admin refund_batch entrypoint to zero disputed usage across services"
labels: type:feature, area:disputes, stack:soroban, stack:rust, priority:low, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Add add an admin refund_batch entrypoint to zero disputed usage across services

### Description
Resolving a widespread incident currently means one `resolve_dispute` call per service. A bounded admin batch refund lets an operator clear a systemic outage in a single transaction.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Accept a bounded `Vec<Symbol>` of services for one agent and apply the same `RefundExceedsUsage` checks per entry.
- Reject oversized batches with the shared batch-length helper.
- Emit one event per refunded pair and add tests for partial failures.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feat/contracts-add-an-admin-refund-batch-entrypoint`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`feat(escrow): add admin refund_batch entrypoint`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Emit dispute lifecycle events for open, cancel, and resolve"
labels: type:feature, area:events, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Add emit dispute lifecycle events for open, cancel, and resolve

### Description
`open_dispute` and `resolve_dispute` change billable state but publish nothing that indexers can follow, unlike the usage and settled emitters. Add a consistent event for each dispute transition.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Use `symbol_short` topics matching the existing emitter naming convention.
- Include agent, service id, and refunded request count in the resolve payload.
- Add tests asserting topics and payloads for each transition.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feat/contracts-emit-dispute-lifecycle-events-for-open`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`feat(escrow): emit dispute lifecycle events`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Extract a shared price-resolution helper unifying flat and tiered lookup"
labels: type:refactor, area:pricing, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Refactor extract a shared price-resolution helper unifying flat and tiered lookup

### Description
Price resolution logic is duplicated between `compute_billing`, `settle`, and `settle_all`, each deciding between `get_price_tiers` and `get_service_price`. A single helper removes the risk of the paths drifting apart.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Add a private `resolve_price(env, service_id, usage) -> i128` used by all three call sites.
- Keep saturating arithmetic semantics identical to today's behaviour.
- Assert existing pricing tests pass unchanged after the extraction.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b refactor/contracts-extract-a-shared-price-resolution-helper`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`refactor(escrow): extract shared price resolution helper`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Replace the get_rate_window tuple return with a named RateWindow struct"
labels: type:refactor, area:rate-limit, stack:soroban, stack:rust, priority:low, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Refactor replace the get_rate_window tuple return with a named RateWindow struct

### Description
`get_rate_window` returns a bare `(u64, u32)` whose fields are only explained by a doc comment, forcing every caller to remember the ordering. A named `contracttype` struct makes the ABI self-describing.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Introduce a `RateWindow { window_start, requests_used }` struct.
- Update `get_remaining_in_window` and all tests to use the named fields.
- Note the ABI change in CHANGELOG.md.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b refactor/contracts-replace-the-get-rate-window-tuple`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`refactor(escrow): return a named RateWindow struct`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Move DataKey access behind a typed storage accessor module"
labels: type:refactor, area:storage, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Refactor move DataKey access behind a typed storage accessor module

### Description
Nearly every entrypoint in `lib.rs` calls `env.storage().persistent()` with a hand-built `DataKey`, spreading raw key construction across two thousand lines. Typed accessors localise the storage layout.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Add get/set/remove helpers per `DataKey` variant with concrete return types.
- Replace direct storage calls in entrypoints with the new helpers.
- Keep the on-chain key encoding byte-identical so deployed state stays readable.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b refactor/contracts-move-datakey-access-behind-a-typed`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`refactor(escrow): add typed storage accessors`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Split lib.rs into storage, billing, admin, and dispute modules"
labels: type:refactor, area:structure, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Refactor split lib.rs into storage, billing, admin, and dispute modules

### Description
`contracts/escrow/src/lib.rs` is a single 1,900-line file holding the admin lifecycle, metering, pricing, settlement, and disputes. Splitting it into modules makes review and ownership tractable without changing the contract ABI.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Create `storage.rs`, `billing.rs`, `admin.rs`, and `disputes.rs` re-exported from `lib.rs`.
- Keep the `#[contractimpl]` block intact so the exported interface is unchanged.
- Verify the built WASM exports the same function set before and after.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b refactor/contracts-split-lib-rs-into-storage-billing`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`refactor(escrow): split lib.rs into focused modules`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Replace magic numeric limits in lib.rs with named constants"
labels: type:refactor, area:validation, stack:soroban, stack:rust, priority:low, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Refactor replace magic numeric limits in lib.rs with named constants

### Description
Batch sizes, the 256-byte description cap, and default rate-limit values appear as inline literals across `lib.rs`, so changing a limit means hunting duplicated numbers. Named constants make the policy surface explicit.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Define `const` values at module scope with `///` comments explaining each limit.
- Replace every inline literal with the corresponding constant.
- Reference the constants in the docs rather than restating the numbers.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b refactor/contracts-replace-magic-numeric-limits-in-lib`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`refactor(escrow): replace magic limits with named constants`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Unify dispute storage keys with the existing usage key layout"
labels: type:refactor, area:storage, stack:soroban, stack:rust, priority:low, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Refactor unify dispute storage keys with the existing usage key layout

### Description
Dispute state is keyed separately from the agent-service `UsageRecord` it annotates, so reading both requires two lookups with different key shapes. Aligning the layouts simplifies iteration in `settle_all`.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Use the same `(Address, Symbol)` key shape for dispute entries as for usage entries.
- Provide a migration path or a schema-version bump if the encoding changes.
- Add tests confirming dispute reads still resolve after the change.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b refactor/contracts-unify-dispute-storage-keys-with-the`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`refactor(escrow): align dispute and usage key layouts`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Deduplicate the settle and settle_all billing-and-drain bodies"
labels: type:refactor, area:settlement, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Refactor deduplicate the settle and settle_all billing-and-drain bodies

### Description
`settle` and `settle_all` each compute a bill, drain the counter, stamp `get_last_settlement`, and emit an event. The duplicated body is exactly the kind of code where one path gains a guard and the other does not.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Extract a private `settle_pair` helper returning the billed amount.
- Have both entrypoints delegate to it after their own authorization checks.
- Add a test asserting single and batch settlement produce identical state.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b refactor/contracts-deduplicate-the-settle-and-settle-all`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`refactor(escrow): extract shared settle_pair helper`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Return a structured settlement receipt from settle instead of a bare i128"
labels: type:enhancement, area:settlement, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Enhance return a structured settlement receipt from settle instead of a bare i128

### Description
`settle` returns only the billed `i128`, hiding the requests settled, the price applied, and any fee split. A receipt struct gives callers everything they need without follow-up reads against already-drained state.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Return a `contracttype` receipt with agent, service, requests, unit price, and total.
- Mirror the same shape inside the `settle_all` result entries.
- Record the ABI change in CHANGELOG.md and update affected tests.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feat/contracts-return-a-structured-settlement-receipt-from`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`feat(escrow): return a structured settlement receipt`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Distinguish unset from zero in get_service_price with an Option return"
labels: type:enhancement, area:pricing, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Enhance distinguish unset from zero in get_service_price with an Option return

### Description
`get_service_price` returns `0` both for a free service and for one that was never priced, which is exactly the ambiguity `remove_service_price` was meant to expose. An `Option<i128>` companion read removes the guesswork.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Add `try_get_service_price` returning `Option<i128>` without changing the existing entrypoint.
- Document the zero-versus-unset distinction next to `remove_service_price`.
- Add tests for never-set, explicitly-zero, and removed prices.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feat/contracts-distinguish-unset-from-zero-in-get`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`feat(escrow): add Option-returning service price read`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Add pagination parameters to get_agent_services"
labels: type:enhancement, area:queries, stack:soroban, stack:rust, priority:low, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Enhance add pagination parameters to get_agent_services

### Description
`get_agent_services` returns the whole tracked list in one `Vec`, so an agent with many services can produce a response that is expensive or impossible to return. `get_agent_usage_page` already establishes the pagination pattern to follow.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Add `get_agent_services_page(agent, offset, limit)` mirroring the existing page entrypoint.
- Enforce the shared batch-length limit on `limit`.
- Add tests asserting pages concatenate to the unpaginated result.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feat/contracts-add-pagination-parameters-to-get-agent`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`feat(escrow): paginate get_agent_services`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Make batch size limits admin-configurable at runtime"
labels: type:enhancement, area:config, stack:soroban, stack:rust, priority:low, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Enhance make batch size limits admin-configurable at runtime

### Description
The bounds behind `BatchTooLarge` and `SettleAllTooLarge` are compile-time constants, so tuning them for real ledger costs requires a redeploy. Storing them as admin-settable config matches how rate limits are already handled.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Add setters and getters guarded by the same admin check as `set_max_requests_per_window`.
- Clamp configured values to a compile-time hard maximum.
- Surface the values in `get_contract_config` and emit a config-change event.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feat/contracts-make-batch-size-limits-admin-configurable`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`feat(escrow): make batch limits admin-configurable`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Extend persistent TTL for price-tier and service-metadata entries on access"
labels: type:enhancement, area:storage, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Enhance extend persistent TTL for price-tier and service-metadata entries on access

### Description
Price tiers and `ServiceMetadata` live in persistent storage and can expire on a long-lived deployment, silently reverting a service to unpriced or unowned. Bumping their TTL on read and write keeps active configuration alive.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Call `extend_ttl` in `get_price_tiers`, `get_service_metadata`, and their setters.
- Use shared threshold and extend-to constants consistent with the usage entries.
- Add tests exercising TTL extension in the test environment.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b feat/contracts-extend-persistent-ttl-for-price-tier`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`feat(escrow): extend TTL for pricing and metadata entries`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Document the dispute, refund, and resolution lifecycle"
labels: type:docs, area:disputes, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Document document the dispute, refund, and resolution lifecycle

### Description
Disputes span `open_dispute`, `has_open_dispute`, and `resolve_dispute` plus three error codes (20-22), and none of it is described outside the source. Contributors and integrators need a written state machine.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Document the state transitions, authorization, and effect on `UsageRecord`.
- Include the interaction between an open dispute and `settle` / `settle_all`.
- Add a worked example with concrete request counts and refund amounts.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b docs/contracts-document-the-dispute-refund-and-resolution`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`docs(escrow): document the dispute lifecycle`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Document the tiered pricing model and tier resolution rules"
labels: type:docs, area:pricing, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Document document the tiered pricing model and tier resolution rules

### Description
`set_price_tiers`, `get_price_tiers`, and `remove_price_tiers` introduce a second pricing mode with no written explanation of how a tier is selected or how it interacts with `set_service_price`.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Explain threshold semantics, inclusive versus exclusive boundaries, and the top-tier fallback.
- State precedence between tiered pricing and the flat price explicitly.
- Include a table of the `InvalidPriceTiers` rejection conditions.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b docs/contracts-document-the-tiered-pricing-model-and`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`docs(escrow): document the tiered pricing model`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Document rate-limit window semantics and configuration"
labels: type:docs, area:rate-limit, stack:soroban, stack:rust, priority:low, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Document document rate-limit window semantics and configuration

### Description
The fixed-window limiter is spread across `set_max_requests_per_window`, `set_rate_window_seconds`, `get_rate_window`, and `get_remaining_in_window`, with the rollover behaviour only inferable from the code.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Describe the fixed-window algorithm and its burst behaviour at window edges.
- Document defaults, the disabled-when-zero convention, and `RateLimitExceeded`.
- Add a configuration example sized for a realistic agent workload.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b docs/contracts-document-rate-limit-window-semantics-and`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`docs(escrow): document rate-limit window semantics`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Add a crate-level rustdoc overview with a quickstart example"
labels: type:docs, area:onboarding, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Document add a crate-level rustdoc overview with a quickstart example

### Description
`contracts/escrow/src/lib.rs` has per-function `///` comments but no `//!` crate-level module documentation, so `cargo doc` output opens on a bare symbol list with no orientation.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Add a `//!` header covering the metering, pricing, settlement, and dispute subsystems.
- Include a short init -> register -> record -> settle quickstart snippet.
- Ensure `cargo doc --no-deps` builds without warnings.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b docs/contracts-add-a-crate-level-rustdoc-overview`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`docs(escrow): add crate-level rustdoc overview`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
++++++
---
type: Feature
title: "Document stroop units and the settlement math end to end"
labels: type:docs, area:settlement, stack:soroban, stack:rust, priority:medium, MAYBE REWARDED, GRANTFOX OSS, OFFICIAL CAMPAIGN, Official Campaign | FWC26
assignees: ''
---
## Document document stroop units and the settlement math end to end

### Description
Prices are stored as `price_stroops` and `compute_billing` multiplies by request count into an `i128` with saturating semantics, but the unit conventions and overflow behaviour are undocumented for integrators.

### Requirements and context
- **Repository scope:** Agentpay-Org/Agentpay-contracts only.
- Define the stroop unit and its relationship to a whole asset unit.
- Walk through `compute_billing` and `settle` arithmetic including saturation at `i128::MAX`.
- Document rounding expectations for tiered and any fee-adjusted amounts.

### Suggested execution
- Fork the repo and create a branch
- `git checkout -b docs/contracts-document-stroop-units-and-the-settlement`
- **Write code in:** `contracts/escrow/src/lib.rs`
- **Write comprehensive tests in:** `contracts/escrow/src/test.rs`
- **Add documentation:** README / docs
- Include NatSpec-style `///` comments

### Test and commit
- Run `cargo fmt --all -- --check`, `cargo build`, `cargo test`
- Cover edge cases and failure paths

### Example commit message
`docs(escrow): document stroop units and settlement math`

### Guidelines
- Minimum 95 percent test coverage for impacted modules
- Clear documentation
- **Timeframe: 96 hours.**

### Community & contribution rewards
- 💬 **Join the AgentPay community on Discord:** https://discord.gg/eXvRKkgcv
- ⭐ This is a **GrantFox OSS / Official Campaign** task and **may be rewarded**. When your PR is merged you'll be prompted to rate the project — a **5-star rating** is much appreciated.
