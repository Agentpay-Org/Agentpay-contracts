# Admin model

## Storage

| Key | Type | Description |
| --- | --- | --- |
| `DataKey::Admin` | `Address` | Operational admin address; set once at `init`, rotated via the two-step transfer flow. |
| `DataKey::PendingAdmin` | `Address` | Address proposed by `propose_admin_transfer`, awaiting `accept_admin_transfer`. Absent when no handover is in progress. |

## Invariants

- **Set exactly once at `init`.** `init` panics with `AlreadyInitialized` if `DataKey::Admin` is already present — including a second call with the *same* address. The only way to change the admin after `init` is the two-step transfer flow below.
- **Every admin-gated entrypoint requires the admin's signature.** The shared `require_admin` helper loads the stored admin and calls `admin.require_auth()`, panicking with `NotInitialized` if `init` has not run yet. This is the canonical gate for admin-only entrypoints (`pause`, `set_service_price`, `register_service`, rate-limit/allowlist/blocklist config, `migrate`, etc.) — see [`CONTRIBUTING.md`](../../CONTRIBUTING.md) for the convention new entrypoints follow.
- **Owner-or-admin entrypoints are a distinct, narrower authorization.** `settle`, `settle_all`, and `transfer_service_ownership` accept either the admin *or* the relevant service's `ServiceMetadata.owner` — never an arbitrary caller. The shared `is_owner_or_admin` helper centralises that comparison; each call site still panics with its own pre-existing error code (`settle` uses `NotPendingAdmin`, the other two use `Unauthorized`) to keep the change behavior-preserving.
- **Two-step handover, not a direct write.** `propose_admin_transfer(new_admin)` (admin-gated) stores `new_admin` under `PendingAdmin` without touching `Admin`. Only `accept_admin_transfer(caller)`, signed by `caller` itself, promotes `PendingAdmin` to `Admin` and clears the pending slot. This prevents an admin from locking the contract out by proposing an address whose key nobody controls — the new admin must prove control by signing the acceptance.
- **Self-proposals are rejected.** `propose_admin_transfer` panics with `InvalidAdminProposal` when `new_admin == admin`, a no-op handover that would otherwise silently succeed.
- **Re-proposing overwrites, it doesn't stack.** Calling `propose_admin_transfer` again before an `accept`/`cancel` simply replaces the stored `PendingAdmin`; there is only ever at most one pending proposal.
- **`cancel_admin_transfer` is admin-gated and idempotent.** It clears `PendingAdmin` if present; calling it with nothing pending is a no-op, not an error.
- **Wrong-caller acceptance is rejected, not silently ignored.** `accept_admin_transfer` panics with `NotPendingAdmin` if the caller does not match the stored `PendingAdmin`, and `NoPendingAdminTransfer` if nothing is pending.

## Entrypoints

| Entrypoint | Gate | Effect |
| --- | --- | --- |
| `init(admin)` | `admin.require_auth()`, once only | Sets `Admin`. |
| `get_admin()` | none (read) | Returns `Option<Address>` — `None` before `init`. |
| `propose_admin_transfer(new_admin)` | admin | Sets `PendingAdmin`. |
| `get_pending_admin()` | none (read) | Returns `Option<Address>` — `None` when no handover is in progress. |
| `get_admin_summary()` | none (read) | Returns `AdminSummary { admin, pending_admin }` — both fields in one round trip. |
| `accept_admin_transfer(caller)` | `caller.require_auth()`, must match `PendingAdmin` | Promotes `PendingAdmin` to `Admin`; clears `PendingAdmin`; emits `admin_chg(old_admin, new_admin)`. |
| `cancel_admin_transfer()` | admin | Clears `PendingAdmin` (no-op if absent). |

## Worked example: rotating the admin

```text
1. init(admin_a)                          // Admin = admin_a
2. propose_admin_transfer(admin_b)        // caller: admin_a. PendingAdmin = admin_b
3. get_pending_admin() -> Some(admin_b)
4. accept_admin_transfer(admin_b)         // caller: admin_b (proves key control)
                                           // Admin = admin_b, PendingAdmin cleared
                                           // emits admin_chg(admin_a, admin_b)
5. get_admin() -> Some(admin_b)
```

If step 4 is called by any address other than `admin_b`, it panics with
`NotPendingAdmin` and no state changes.
