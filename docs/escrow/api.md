# Escrow API and Error Reference

This reference catalogs every public `Escrow` entrypoint exposed from
`contracts/escrow/src/lib.rs`, including authorization, pause behavior,
parameters, return values, and contract errors. Error codes are append-only:
new variants should be added after the current highest code and existing
numeric meanings must not be renumbered.

## Shared types

| Type | Fields | Notes |
| --- | --- | --- |
| `UsageRecord` | `agent: Address`, `service_id: Symbol`, `requests: u32` | Returned by `record_usage`; `requests` is the new accumulated total, not the delta. |
| `ServiceMetadata` | `description: String`, `owner: Address` | Stored per service for dashboard and settlement reporting. |

## Entrypoint reference

| Entrypoint | Auth | Pause behavior | Parameters | Returns | Errors |
| --- | --- | --- | --- | --- | --- |
| `init(env, admin)` | `admin.require_auth()` | Allowed while paused state is absent; can only run before initialization. | `admin: Address` | `()` | `AlreadyInitialized (#1)` |
| `get_admin(env)` | None | Read-only; not blocked by pause. | None | `Option<Address>` | None |
| `record_usage(env, agent, service_id, requests)` | None | Rejected while paused. | `agent: Address`, `service_id: Symbol`, `requests: u32` | `UsageRecord` | `ContractPaused (#4)`, `RequestsMustBePositive (#2)`, `RequestsExceedsMaxPerCall (#8)`, `RequestsBelowMinPerCall (#9)`, `ServiceNotRegistered (#7)`, `ServiceDisabled (#12)`, `AgentNotAllowed (#10)` |
| `get_last_settlement(env, agent, service_id)` | None | Read-only; not blocked by pause. | `agent: Address`, `service_id: Symbol` | `Option<u64>` | None |
| `get_total_requests_all_time(env)` | None | Read-only; not blocked by pause. | None | `u64` | None |
| `get_total_usage_by_agent(env, agent)` | None | Read-only; not blocked by pause. | `agent: Address` | `u32` | None |
| `get_usage(env, agent, service_id)` | None | Read-only; not blocked by pause. | `agent: Address`, `service_id: Symbol` | `u32` | None |
| `set_service_price(env, service_id, price_stroops)` | Stored admin | Admin configuration; not blocked by pause. | `service_id: Symbol`, `price_stroops: i128` | `()` | `NotInitialized (#3)`, `RequestsMustBePositive (#2)` for negative prices |
| `get_service_price(env, service_id)` | None | Read-only; not blocked by pause. | `service_id: Symbol` | `i128` | None |
| `compute_billing(env, agent, service_id)` | None | Read-only; not blocked by pause. | `agent: Address`, `service_id: Symbol` | `i128` | None |
| `settle(env, agent, service_id)` | Stored admin | Rejected while paused. | `agent: Address`, `service_id: Symbol` | `i128` billed amount in stroops | `ContractPaused (#4)`, `NotInitialized (#3)` |
| `get_min_requests_per_call(env)` | None | Read-only; not blocked by pause. | None | `u32` | None |
| `set_allowlist_enabled(env, enabled)` | Stored admin | Admin configuration; not blocked by pause. | `enabled: bool` | `()` | `NotInitialized (#3)` |
| `is_allowlist_enabled(env)` | None | Read-only; not blocked by pause. | None | `bool` | None |
| `is_agent_allowed(env, agent)` | None | Read-only; not blocked by pause. | `agent: Address` | `bool` | None |
| `set_agent_allowed(env, agent, allowed)` | Stored admin | Admin configuration; not blocked by pause. | `agent: Address`, `allowed: bool` | `()` | `NotInitialized (#3)` |
| `set_min_requests_per_call(env, min_requests)` | Stored admin | Admin configuration; not blocked by pause. | `min_requests: u32` | `()` | `NotInitialized (#3)` |
| `get_max_requests_per_call(env)` | None | Read-only; not blocked by pause. | None | `u32` | None |
| `set_max_requests_per_call(env, max_requests)` | Stored admin | Admin configuration; not blocked by pause. | `max_requests: u32` | `()` | `NotInitialized (#3)` |
| `set_require_service_registration(env, required)` | Stored admin | Admin configuration; not blocked by pause. | `required: bool` | `()` | `NotInitialized (#3)` |
| `is_service_registration_required(env)` | None | Read-only; not blocked by pause. | None | `bool` | None |
| `is_service_registered(env, service_id)` | None | Read-only; not blocked by pause. | `service_id: Symbol` | `bool` | None |
| `unregister_service(env, service_id)` | Stored admin | Admin configuration; not blocked by pause. | `service_id: Symbol` | `()` | `NotInitialized (#3)` |
| `register_service(env, service_id)` | Stored admin | Admin configuration; not blocked by pause. | `service_id: Symbol` | `()` | `NotInitialized (#3)` |
| `cancel_admin_transfer(env)` | Stored admin | Admin configuration; not blocked by pause. | None | `()` | `NotInitialized (#3)` |
| `get_pending_admin(env)` | None | Read-only; not blocked by pause. | None | `Option<Address>` | None |
| `accept_admin_transfer(env, caller)` | `caller.require_auth()`; caller must equal the pending admin. | Admin handover; not blocked by pause. | `caller: Address` | `()` | `NoPendingAdminTransfer (#5)`, `NotPendingAdmin (#6)` |
| `propose_admin_transfer(env, new_admin)` | Stored admin | Admin configuration; not blocked by pause. | `new_admin: Address` | `()` | `NotInitialized (#3)` |
| `is_paused(env)` | None | Read-only. | None | `bool` | None |
| `unpause(env)` | Stored admin | Explicitly clears pause state. | None | `()` | `NotInitialized (#3)` |
| `pause(env)` | Stored admin | Explicitly sets pause state. | None | `()` | `NotInitialized (#3)` |
| `migrate_v1_to_v2(env)` | Stored admin | Migration admin call; not blocked by pause. | None | `()` | `NotInitialized (#3)`, `MigrationVersionMismatch (#11)` |
| `get_service_metadata(env, service_id)` | None | Read-only; not blocked by pause. | `service_id: Symbol` | `Option<ServiceMetadata>` | None |
| `is_service_disabled(env, service_id)` | None | Read-only; not blocked by pause. | `service_id: Symbol` | `bool` | None |
| `set_service_disabled(env, service_id, disabled)` | Stored admin | Admin configuration; not blocked by pause. | `service_id: Symbol`, `disabled: bool` | `()` | `NotInitialized (#3)` |
| `set_service_metadata(env, service_id, description, owner)` | Stored admin | Admin configuration; not blocked by pause. | `service_id: Symbol`, `description: String`, `owner: Address` | `()` | `NotInitialized (#3)` |
| `get_schema_version(env)` | None | Read-only; not blocked by pause. | None | `u32` | None |
| `version(env)` | None | Read-only; not blocked by pause. | None | `u32` | None |

## Error code catalog

| Code | Variant | Trigger condition | Raised by |
| --- | --- | --- | --- |
| 1 | `AlreadyInitialized` | `init` is called after an admin has already been stored. | `init` |
| 2 | `RequestsMustBePositive` | `record_usage` receives `requests == 0`, or `set_service_price` receives a negative price. | `record_usage`, `set_service_price` |
| 3 | `NotInitialized` | An admin-gated entrypoint tries to load `DataKey::Admin` before `init` has stored it. | `set_service_price`, `settle`, `set_allowlist_enabled`, `set_agent_allowed`, `set_min_requests_per_call`, `set_max_requests_per_call`, `set_require_service_registration`, `unregister_service`, `register_service`, `cancel_admin_transfer`, `propose_admin_transfer`, `unpause`, `pause`, `migrate_v1_to_v2`, `set_service_disabled`, `set_service_metadata` |
| 4 | `ContractPaused` | A state-changing metering or settlement entrypoint is called while `Paused` is true. | `record_usage`, `settle` |
| 5 | `NoPendingAdminTransfer` | `accept_admin_transfer` is called when `PendingAdmin` is absent. | `accept_admin_transfer` |
| 6 | `NotPendingAdmin` | `accept_admin_transfer` is called by an address that does not match the pending admin. | `accept_admin_transfer` |
| 7 | `ServiceNotRegistered` | Strict service registration is enabled and the service ID is not registered. | `record_usage` |
| 8 | `RequestsExceedsMaxPerCall` | `requests` is greater than the configured `MaxRequestsPerCall`. | `record_usage` |
| 9 | `RequestsBelowMinPerCall` | `requests` is lower than the configured `MinRequestsPerCall`. | `record_usage` |
| 10 | `AgentNotAllowed` | The allowlist is enabled and the agent is absent or false in `AgentAllowed`. | `record_usage` |
| 11 | `MigrationVersionMismatch` | `migrate_v1_to_v2` sees a schema version other than `1`. | `migrate_v1_to_v2` |
| 12 | `ServiceDisabled` | The service has been disabled with `set_service_disabled`. | `record_usage` |

## Event surface

Only three entrypoints emit events:

| Event topic | Emitted by | Payload | Meaning |
| --- | --- | --- | --- |
| `usage` | `record_usage` | `(agent, service_id, requests_delta, new_total)` | A usage delta was accepted and accumulated. |
| `settled` | `settle` | `(agent, service_id, drained_requests, billed_stroops)` | Outstanding usage was priced and drained to zero. |
| `paused` | `pause`, `unpause` | `true` or `false` | The pause flag changed. |

Configuration entrypoints such as service registration, pricing, metadata,
allowlist updates, and service disabling write persistent storage but do not
emit events. Indexers should pair the events above with read methods when they
need the current configuration.

## Default values

Several read methods return explicit defaults when a storage key is absent:

| Read method | Default |
| --- | --- |
| `get_usage` | `0` |
| `get_service_price` | `0` |
| `compute_billing` | `0` when usage or price is absent |
| `get_min_requests_per_call` | `0` |
| `get_max_requests_per_call` | `u32::MAX` |
| `is_allowlist_enabled` | `false` |
| `is_agent_allowed` | `false` |
| `is_service_registration_required` | `false` |
| `is_service_registered` | `false` |
| `is_paused` | `false` |
| `is_service_disabled` | `false` |
| `get_schema_version` | `1` |

These defaults preserve pre-configuration behavior until an admin explicitly
enables stricter controls.

## Contributor notes

- Keep `EscrowError` numeric codes append-only. Do not reuse or renumber a code
  once released.
- When adding a public entrypoint, update the entrypoint reference and any
  relevant error rows in this file in the same change.
- When adding a new event, update the event surface table with its topic,
  payload, and emitting entrypoint.
