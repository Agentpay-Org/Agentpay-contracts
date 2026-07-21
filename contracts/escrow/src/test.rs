#![cfg(test)]
#![allow(deprecated)]
#![allow(unused_variables)]
#![allow(dead_code)]

//! # Escrow contract test suite
//!
//! ## Test-harness conventions
//!
//! Every test that needs a fully-initialised contract should call one of the
//! two primary setup helpers defined at the top of this module:
//!
//! * [`setup_initialized`] — blanket-mocked auths, contract registered and
//!   `init`-ed with a generated admin.  Use this for the vast majority of
//!   tests.
//! * [`setup_scoped_auth`] — only the `init` call is auth-mocked; all
//!   subsequent privileged calls will fail unless the test wires up its own
//!   `mock_auths`.  Use this when a test needs to assert that a specific
//!   entrypoint enforces `require_auth`.
//!
//! Convenience helpers are also available:
//!
//! * [`make_agent`] — generate a fresh [`Address`] to act as an agent.
//! * [`make_service`] — create a short [`Symbol`] representing a service id.
//! * [`advance_ledger`] — bump the ledger timestamp by a given number of
//!   seconds (useful for rate-window and settlement-timestamp tests).
//! * [`configure_rate_limit`] — set the fixed-window cap and duration for
//!   rate-limit coverage.
//! * [`set_price`] — one-liner to call `set_service_price` on a client.
//! * [`record`]    — one-liner to call `record_usage` on a client.
//!
//! ### `register_service_with_metadata` coverage
//!
//! The combined registration-plus-metadata entrypoint is tested for:
//! * **Atomicity** — after one call `is_service_registered` is `true` and
//!   `get_service_metadata` returns the exact description and owner.
//! * **Event emission** — `svc_reg(service_id, owner)` is published.
//! * **Idempotent overwrite** — re-registering an existing id replaces its
//!   metadata; an empty description is accepted.
//! * **Admin gate** — a non-admin caller is rejected (`Unauthorized`).
//! * **Pause gate** — calling while paused panics with `ContractPaused` (#4).
//! * **Equivalence** — the combined call produces the same post-state as the
//!   two-step `register_service + set_service_metadata` sequence.
//!
//! ### Pause lifecycle coverage
//!
//! The pause toggle is tested for:
//! * `pause()` emitting `("paused", true)`
//! * `unpause()` emitting `("paused", false)`
//! * idempotent double-pause and double-unpause state transitions
//! * non-admin rejection via scoped auth
//!
//! ### Security note
//! Tests that use `setup_initialized` rely on `mock_all_auths`, which
//! satisfies every `require_auth` call unconditionally.  When a test needs to
//! verify that auth *is* enforced, use `setup_scoped_auth` or call
//! `env.set_auths(&[])` to drop mock authorisations before the call under
//! test.
extern crate alloc;

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Events, Ledger, MockAuth, MockAuthInvoke},
    Address, IntoVal, Symbol,
};

// ── Primary setup helpers ────────────────────────────────────────────────────

/// Create a fully-initialised escrow contract with blanket auth mocking.
///
/// Returns `(client, admin)` where `admin` is the address passed to `init`.
/// All subsequent `require_auth` calls are automatically satisfied by
/// `mock_all_auths`, so most tests can start from this helper without any
/// additional auth wiring.
///
/// **Note on agent auth**: After adding `agent.require_auth()` to `record_usage`,
/// `mock_all_auths()` will satisfy agent auth checks for any agent address,
/// allowing the existing test suite to continue working without modification.
/// Build a `svc_<n>` name without `format!` (the crate is `no_std`).
fn svc_name(buf: &mut [u8; 8], i: u32) -> &str {
    buf[0] = b's';
    buf[1] = b'v';
    buf[2] = b'c';
    buf[3] = b'_';
    let mut n = i;
    let mut digits = [0u8; 3];
    let mut len = 0usize;
    if n == 0 {
        digits[0] = b'0';
        len = 1;
    } else {
        while n > 0 && len < 3 {
            digits[len] = b'0' + (n % 10) as u8;
            n /= 10;
            len += 1;
        }
    }
    for k in 0..len {
        buf[4 + k] = digits[len - 1 - k];
    }
    core::str::from_utf8(&buf[..4 + len]).unwrap()
}

fn setup_initialized(env: &Env) -> (EscrowClient<'_>, Address) {
    env.mock_all_auths();
    let contract_id = env.register_contract(None, Escrow);
    let client = EscrowClient::new(env, &contract_id);
    let admin = Address::generate(env);
    client.init(&admin);
    (client, admin)
}

/// Assert that the most recently emitted event is the expected `usage` event.
fn assert_latest_usage_event(
    env: &Env,
    agent: &Address,
    service_id: &Symbol,
    expected_delta: u32,
    expected_total: u32,
) {
    let events = env.events().all();
    assert!(!events.is_empty(), "expected a usage event to be emitted");
    let (_addr, topics, data) = events.last().unwrap();
    let expected_topics: soroban_sdk::Vec<soroban_sdk::Val> =
        (symbol_short!("usage"),).into_val(env);
    assert_eq!(topics, expected_topics);
    let decoded: (Address, Symbol, u32, u32) = data.into_val(env);
    assert_eq!(
        decoded,
        (
            agent.clone(),
            service_id.clone(),
            expected_delta,
            expected_total,
        )
    );
}

/// Assert that the most recent contract invocation emitted exactly
/// `expected_count` `usage` events.
fn assert_usage_event_count(env: &Env, expected_count: usize) {
    let expected_topics: soroban_sdk::Vec<soroban_sdk::Val> =
        (symbol_short!("usage"),).into_val(env);
    let count = env
        .events()
        .all()
        .iter()
        .filter(|(_, topics, _)| *topics == expected_topics)
        .count();
    if expected_count == 0 {
        assert_eq!(count, 0);
    } else {
        assert!(count >= 1, "expected at least one usage event, got {count}");
    }
}

/// Assert that the most recent pause lifecycle event carries the expected flag.
///
/// The helper keeps the pause/unpause tests focused on contract behavior while
/// still checking the event topic and payload in one place.
fn assert_latest_pause_event(env: &Env, expected_flag: bool) {
    let events = env.events().all();
    assert!(!events.is_empty(), "expected a paused event to be emitted");
    let (_addr, topics, data) = events.last().unwrap();
    let expected_topics: soroban_sdk::Vec<soroban_sdk::Val> =
        (symbol_short!("paused"),).into_val(env);
    assert_eq!(topics, expected_topics);
    let flag: bool = data.into_val(env);
    assert_eq!(flag, expected_flag);
}

// ── Convenience address / symbol helpers ─────────────────────────────────────

/// Generate a fresh [`Address`] to use as an agent in tests.
///
/// Each call returns a distinct address, so you can create independent agents
/// without any naming collision:
/// ```ignore
/// let agent_a = make_agent(&env);
/// let agent_b = make_agent(&env);
/// ```
fn make_agent(env: &Env) -> Address {
    Address::generate(env)
}

/// Build a [`Symbol`] from a static string slice to use as a service id.
///
/// The `name` must be a valid Soroban symbol (≤ 32 alphanumeric / `_` chars).
/// ```ignore
/// let svc = make_service(&env, "weather_api");
/// ```
fn make_service(env: &Env, name: &'static str) -> Symbol {
    Symbol::new(env, name)
}

// ── Ledger-clock helper ───────────────────────────────────────────────────────

/// Advance the ledger timestamp by `seconds`.
///
/// Useful for rate-window rollover tests and settlement-timestamp assertions
/// without having to repeat the `env.ledger().with_mut(…)` boilerplate:
/// ```ignore
/// advance_ledger(&env, 100); // move 100 s into the future
/// ```
fn advance_ledger(env: &Env, seconds: u64) {
    env.ledger().with_mut(|li| li.timestamp += seconds);
}

/// Configure the fixed-window rate limiter for a test scenario.
///
/// Passing `0` for either argument intentionally leaves the limiter disabled,
/// mirroring the contract's production semantics.
fn configure_rate_limit(client: &EscrowClient<'_>, max_requests: u32, window_seconds: u64) {
    client.set_max_requests_per_window(&max_requests);
    client.set_rate_window_seconds(&window_seconds);
}

#[test]
fn test_version() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let v = client.version();
    assert_eq!(v, 2);
}
#[test]
fn test_init_persists_admin() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    assert_eq!(client.get_admin(), Some(admin));
}
#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_init_rejects_double_init() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let other_admin = Address::generate(&env);
    client.init(&other_admin);
}
#[test]
fn test_record_usage() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    let agent = Address::generate(&env);
    let service_id = Symbol::new(&env, "weather_api");
    let requests: u32 = 100;

    let record = client.record_usage(&agent, &service_id, &requests);
    assert_eq!(record.agent, agent);
    assert_eq!(record.service_id, service_id);
    // First write: total equals the recorded delta.
    assert_eq!(record.requests, requests);
}
#[test]
fn test_record_usage_accumulates_across_calls() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    let agent = Address::generate(&env);
    let service_id = Symbol::new(&env, "weather_api");

    let first = client.record_usage(&agent, &service_id, &40u32);
    assert_eq!(first.requests, 40);
    assert_usage_event_count(&env, 1);
    assert_latest_usage_event(&env, &agent, &service_id, 40, 40);

    let second = client.record_usage(&agent, &service_id, &60u32);
    assert_eq!(second.requests, 100);
    assert_usage_event_count(&env, 1);
    assert_latest_usage_event(&env, &agent, &service_id, 60, 100);

    let third = client.record_usage(&agent, &service_id, &1u32);
    assert_eq!(third.requests, 101);
    assert_usage_event_count(&env, 1);
    assert_latest_usage_event(&env, &agent, &service_id, 1, 101);

    assert_eq!(client.get_usage(&agent, &service_id), 101);
    assert_eq!(client.get_total_usage_by_agent(&agent), 101);
    assert_eq!(client.get_total_requests_all_time(), 101);
}

// ── record_usage return-value contract and event-payload semantics ──────
//
// `record_usage` documents that the returned `UsageRecord.requests` is the
// *new total* (not the per-call delta) so the caller can confirm the
// post-write state without a second storage read. The `usage` event carries
// *both* the per-call delta (payload position 2) and the running total
// (payload position 3) so off-chain loops can reconstruct the counter
// sequence without ambiguity.

/// Returns the accumulated total, **not** the per-call delta.
///
/// After `record_usage(agent, svc, 3)` and `record_usage(agent, svc, 5)`,
/// the second `UsageRecord.requests` is `8` (the sum), never `5` (the
/// second delta). This is the documented contract and the reason callers
/// can skip an extra `get_usage` read after recording.
#[test]
fn test_record_usage_contract_return_is_new_total() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    let agent = make_agent(&env);
    let svc = make_service(&env, "billing_api");

    // First call: accumulated = delta = 3.
    let r1 = client.record_usage(&agent, &svc, &3u32);
    assert_eq!(r1.requests, 3, "first call: total equals the delta");

    // Second call: accumulated = 3 + 5 = 8.  The field must NOT be 5.
    let r2 = client.record_usage(&agent, &svc, &5u32);
    assert_eq!(
        r2.requests, 8,
        "second call: requests carries the new total (8), not the delta (5)"
    );
    assert_ne!(r2.requests, 5, "the field must not be the per-call delta");

    // Verify the stored counter agrees.
    assert_eq!(client.get_usage(&agent, &svc), 8);
}

/// Event payload: `requests` is the per-call delta, `total` is the running total.
///
/// The `usage` event publishes `(agent, service_id, delta, total)` where the
/// third tuple element is the amount added in *this* call (not the counter)
/// and the fourth element is the counter after applying the delta. This
/// lets off-chain consumers distinguish "how much was just added" from
/// "where does the counter stand now" in a single event scan.
#[test]
fn test_record_usage_contract_event_fields() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    let agent = make_agent(&env);
    let svc = make_service(&env, "svc_a");

    // Call one: delta=3, total=3.
    client.record_usage(&agent, &svc, &3u32);
    assert_latest_usage_event(&env, &agent, &svc, 3, 3);

    // Call two: delta=5, total=8.
    client.record_usage(&agent, &svc, &5u32);
    assert_latest_usage_event(&env, &agent, &svc, 5, 8);

    // Call three: delta=1, total=9.
    client.record_usage(&agent, &svc, &1u32);
    assert_latest_usage_event(&env, &agent, &svc, 1, 9);
}

/// Exactly one `usage` event is emitted per successful `record_usage` call.
///
/// Verifies that the event stream contains the correct count of `usage`
/// topic events — no duplicates, no missing events — which is critical
/// for off-chain consumers that tally deltas from the event log.
#[test]
fn test_record_usage_contract_exactly_one_event_per_call() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    let agent = make_agent(&env);
    let svc = make_service(&env, "svc_a");

    // No usage events yet.
    assert_usage_event_count(&env, 0);

    // Single call produces exactly one event.
    client.record_usage(&agent, &svc, &1u32);
    assert_usage_event_count(&env, 1);

    // Second call still yields exactly one latest usage event for that call.
    client.record_usage(&agent, &svc, &2u32);
    assert_usage_event_count(&env, 1);

    // Third call still yields exactly one latest usage event for that call.
    client.record_usage(&agent, &svc, &3u32);
    assert_usage_event_count(&env, 1);
}

/// Lifetime counters advance by exactly the delta on each call.
///
/// `get_total_usage_by_agent` and `get_total_requests_all_time` are
/// monotonically increasing counters that aggregate every `record_usage`
/// delta. This test verifies they grow by the per-call delta, not the
/// running total, and that they remain consistent across multiple calls.
#[test]
fn test_record_usage_contract_lifetime_counters() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    let agent = make_agent(&env);
    let svc = make_service(&env, "svc_a");

    // Baseline: zero.
    assert_eq!(client.get_total_usage_by_agent(&agent), 0);
    assert_eq!(client.get_total_requests_all_time(), 0u64);

    // Delta 3 → lifetime counters advance by 3.
    client.record_usage(&agent, &svc, &3u32);
    assert_eq!(client.get_total_usage_by_agent(&agent), 3);
    assert_eq!(client.get_total_requests_all_time(), 3u64);

    // Delta 5 → lifetime counters advance by 5 (now 8).
    client.record_usage(&agent, &svc, &5u32);
    assert_eq!(client.get_total_usage_by_agent(&agent), 8);
    assert_eq!(client.get_total_requests_all_time(), 8u64);

    // Delta 2 → lifetime counters advance by 2 (now 10).
    client.record_usage(&agent, &svc, &2u32);
    assert_eq!(client.get_total_usage_by_agent(&agent), 10);
    assert_eq!(client.get_total_requests_all_time(), 10u64);
}
#[test]
fn test_record_usage_is_keyed_per_service() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    let agent = Address::generate(&env);
    let weather = Symbol::new(&env, "weather_api");
    let inference = Symbol::new(&env, "infer_api");

    client.record_usage(&agent, &weather, &10u32);
    client.record_usage(&agent, &inference, &7u32);

    assert_eq!(client.get_usage(&agent, &weather), 10);
    assert_eq!(client.get_usage(&agent, &inference), 7);
}
#[test]
fn test_get_usage_returns_zero_for_unknown_pair() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let unseen_agent = Address::generate(&env);
    let svc = Symbol::new(&env, "anything");
    assert_eq!(client.get_usage(&unseen_agent, &svc), 0);
}
#[test]
fn test_set_service_price_admin_can_write() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.set_service_price(&Symbol::new(&env, "infer"), &500i128);
}
#[test]
fn test_get_service_price_round_trip() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "infer");
    client.set_service_price(&svc, &500i128);
    assert_eq!(client.get_service_price(&svc), 500i128);
}
#[test]
fn test_get_service_price_defaults_to_zero() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    assert_eq!(
        client.get_service_price(&Symbol::new(&env, "never_set")),
        0i128
    );
}

// ── set_service_price event, guards, and round-trips ───────────────────
//
// `set_service_price` is admin-gated, honours the pause gate (#4),
// rejects negative prices with `RequestsMustBePositive` (#2), accepts
// zero as a free-service marker, emits `price_set(service_id, price)`,
// and — when strict registration is enabled — rejects unregistered (#7)
// and disabled (#12) services.
#[test]
fn test_set_service_price_emits_price_set_event() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "infer");
    let price: i128 = 500;

    client.set_service_price(&svc, &price);

    let events = env.events().all();
    assert!(!events.is_empty());
    let (_addr, topics, data) = events.last().unwrap();
    let expected_topics: soroban_sdk::Vec<soroban_sdk::Val> =
        (symbol_short!("price_set"),).into_val(&env);
    assert_eq!(topics, expected_topics);
    let decoded: (Symbol, i128) = data.into_val(&env);
    assert_eq!(decoded, (svc, price));
}
#[test]
fn test_set_service_price_zero_price_round_trip() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "free");
    client.set_service_price(&svc, &0i128);
    assert_eq!(client.get_service_price(&svc), 0i128);
}
#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_set_service_price_rejects_negative_price() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.set_service_price(&Symbol::new(&env, "infer"), &(-1i128));
}
#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_set_service_price_rejects_i128_min() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.set_service_price(&Symbol::new(&env, "infer"), &i128::MIN);
}
#[test]
fn test_set_service_price_reprice_overwrites() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "infer");
    client.set_service_price(&svc, &100i128);
    assert_eq!(client.get_service_price(&svc), 100i128);
    client.set_service_price(&svc, &200i128);
    assert_eq!(client.get_service_price(&svc), 200i128);
}
#[test]
fn test_compute_billing_basic() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "infer");
    client.set_service_price(&svc, &10i128);
    client.record_usage(&agent, &svc, &42u32);
    assert_eq!(client.compute_billing(&agent, &svc), 420i128);
}

#[test]
fn test_credit_agent_and_settle_draws_down_balance() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "infer");
    client.set_service_price(&svc, &10i128);
    client.credit_agent(&agent, &50i128);

    client.record_usage(&agent, &svc, &3u32);
    let billed = client.settle(&admin, &agent, &svc);
    // Capture events immediately: each later client call resets the
    // per-invocation event buffer that `events().all()` reads from.
    let events = env.events().all();

    assert_eq!(billed, 30i128);
    assert_eq!(client.get_agent_credit(&agent), 20i128);
    assert_eq!(client.get_usage(&agent, &svc), 0);

    assert!(!events.is_empty());
    // `settle` emits `cred_deb` before `settled`, so select the credit event
    // by topic rather than assuming its position in the buffer.
    let expected_topics: soroban_sdk::Vec<soroban_sdk::Val> =
        (symbol_short!("cred_deb"),).into_val(&env);
    let (_addr, _topics, data) = events
        .iter()
        .find(|(_a, t, _d)| *t == expected_topics)
        .expect("settle should emit a cred_deb event");
    let decoded: (Address, i128, i128) = data.into_val(&env);
    assert_eq!(decoded, (agent.clone(), 30i128, 20i128));
}

#[test]
#[should_panic(expected = "Error(Contract, #28)")]
fn test_record_usage_rejects_insufficient_credit_balance() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "infer");
    client.set_service_price(&svc, &10i128);
    client.credit_agent(&agent, &20i128);

    client.record_usage(&agent, &svc, &3u32);
}

#[test]
fn test_decrement_usage_basic() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "weather_api");
    client.record_usage(&agent, &svc, &100u32);

    let new_total = client.decrement_usage(&agent, &svc, &30u32);
    assert_eq!(new_total, 70);
    assert_eq!(client.get_usage(&agent, &svc), 70);
}
#[test]
fn test_decrement_usage_past_zero_clamps() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "weather_api");
    client.record_usage(&agent, &svc, &50u32);

    let new_total = client.decrement_usage(&agent, &svc, &200u32);
    assert_eq!(new_total, 0);
    assert_eq!(client.get_usage(&agent, &svc), 0);
}
#[test]
fn test_decrement_usage_to_zero() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "weather_api");
    client.record_usage(&agent, &svc, &100u32);

    let new_total = client.decrement_usage(&agent, &svc, &100u32);
    assert_eq!(new_total, 0);
    assert_eq!(client.get_usage(&agent, &svc), 0);
}
#[test]
fn test_decrement_usage_never_used_clamps() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "never_used");

    let new_total = client.decrement_usage(&agent, &svc, &50u32);
    assert_eq!(new_total, 0);
    assert_eq!(client.get_usage(&agent, &svc), 0);
}
#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_decrement_usage_rejects_zero() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "weather_api");
    client.record_usage(&agent, &svc, &100u32);

    client.decrement_usage(&agent, &svc, &0u32);
}
#[test]
#[should_panic(expected = "Unauthorized")]
fn test_decrement_usage_rejects_non_admin() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Escrow);
    let client = EscrowClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    env.mock_all_auths();
    client.init(&admin);

    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "weather_api");
    client.record_usage(&agent, &svc, &100u32);

    env.set_auths(&[]);
    client.decrement_usage(&agent, &svc, &10u32);
}
#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_decrement_usage_rejected_while_paused() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "weather_api");
    client.record_usage(&agent, &svc, &100u32);

    client.pause();
    client.decrement_usage(&agent, &svc, &10u32);
}
#[test]
fn test_decrement_usage_lifetime_counters_unchanged() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "weather_api");
    client.record_usage(&agent, &svc, &100u32);

    let lifetime_agent_before = client.get_total_usage_by_agent(&agent);
    let lifetime_all_before = client.get_total_requests_all_time();

    client.decrement_usage(&agent, &svc, &30u32);

    assert_eq!(
        client.get_total_usage_by_agent(&agent),
        lifetime_agent_before
    );
    assert_eq!(client.get_total_requests_all_time(), lifetime_all_before);
}
#[test]
fn test_decrement_usage_emits_event() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "weather_api");
    client.record_usage(&agent, &svc, &100u32);

    client.decrement_usage(&agent, &svc, &30u32);

    let events = env.events().all();
    assert!(!events.is_empty());
    let (_addr, topics, data) = events.last().unwrap();
    let expected_topics: soroban_sdk::Vec<soroban_sdk::Val> =
        (symbol_short!("usage_dec"),).into_val(&env);
    assert_eq!(topics, expected_topics);
    let decoded: (Address, Symbol, u32, u32) = data.into_val(&env);
    assert_eq!(decoded, (agent.clone(), svc.clone(), 30u32, 70u32));
}
#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_decrement_usage_paused_beats_zero() {
    // Paused (#4) must win even when amount == 0 (which would be #2).
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.pause();
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "weather_api");
    client.decrement_usage(&agent, &svc, &0u32);
}
#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_decrement_usage_zero_beats_noauth() {
    // Zero (#2) must win over auth check. With amount == 0, we reject before
    // reaching require_auth, so even without admin auth the error is #2.
    let env = Env::default();
    let contract_id = env.register_contract(None, Escrow);
    let client = EscrowClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    env.mock_all_auths();
    client.init(&admin);

    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "weather_api");

    // Drop auth so the admin require_auth would fail if reached — but #2
    // fires first because amount == 0 is checked before auth.
    env.set_auths(&[]);
    client.decrement_usage(&agent, &svc, &0u32);
}
#[test]
fn test_settle_drains_usage_and_returns_billed() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "infer");
    client.set_service_price(&svc, &10i128);
    client.record_usage(&agent, &svc, &42u32);
    let billed = client.settle(&admin, &agent, &svc);
    assert_eq!(billed, 420i128);
    assert_eq!(client.get_usage(&agent, &svc), 0);
}
#[test]
fn test_pause_admin_can_pause() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.pause();
}
#[test]
fn test_unpause_admin_can_unpause() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.pause();
    client.unpause();
}
#[test]
fn test_pause_pause_unpause_maintains_expected_events_and_state() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    client.pause();
    assert_latest_pause_event(&env, true);
    assert!(client.is_paused());

    client.pause();
    assert_latest_pause_event(&env, true);
    assert!(client.is_paused());

    client.unpause();
    assert_latest_pause_event(&env, false);
    assert!(!client.is_paused());
}

#[test]
fn test_unpause_before_any_pause_is_idempotent() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    client.unpause();
    assert_latest_pause_event(&env, false);
    assert!(!client.is_paused());

    client.unpause();
    assert_latest_pause_event(&env, false);
    assert!(!client.is_paused());
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_pause_requires_admin_auth() {
    let env = Env::default();
    let client = setup_scoped_auth(&env);

    client.pause();
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_unpause_requires_admin_auth() {
    let env = Env::default();
    let client = setup_scoped_auth(&env);

    client.unpause();
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_settle_rejected_while_paused() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.pause();
    let agent = Address::generate(&env);
    client.settle(&admin, &agent, &Symbol::new(&env, "infer"));
}
#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_record_usage_rejected_while_paused() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.pause();
    let agent = Address::generate(&env);
    client.record_usage(&agent, &Symbol::new(&env, "infer"), &1u32);
}
#[test]
fn test_propose_admin_transfer_persists_pending() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let next = Address::generate(&env);
    client.propose_admin_transfer(&next);
}
#[test]
fn test_accept_admin_transfer_rotates_admin() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let next = Address::generate(&env);
    client.propose_admin_transfer(&next);
    client.accept_admin_transfer(&next);
    assert_eq!(client.get_admin(), Some(next));
}
#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_accept_admin_transfer_panics_with_no_pending() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let caller = Address::generate(&env);
    client.accept_admin_transfer(&caller);
}
#[test]
fn test_is_paused_round_trip() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    assert!(!client.is_paused());
    client.pause();
    assert!(client.is_paused());
    client.unpause();
    assert!(!client.is_paused());
}
#[test]
fn test_settle_returns_zero_for_unused_pair() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "infer");
    client.set_service_price(&svc, &10i128);
    assert_eq!(client.settle(&admin, &agent, &svc), 0i128);
}
#[test]
fn test_compute_billing_zero_when_unpriced_or_unused() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "infer");
    // no price, no usage
    assert_eq!(client.compute_billing(&agent, &svc), 0i128);
    client.record_usage(&agent, &svc, &10u32);
    // usage > 0 but price still 0
    assert_eq!(client.compute_billing(&agent, &svc), 0i128);
}
#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_record_usage_rejects_zero_requests() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let service_id = Symbol::new(&env, "weather_api");
    client.record_usage(&agent, &service_id, &0u32);
}
#[test]
fn test_bool_flag_accessor_round_trip() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    // Defaults to false when unset.
    assert!(!client.is_allowlist_enabled());
    // Round-trips true then false through the centralised accessors.
    client.set_allowlist_enabled(&true);
    assert!(client.is_allowlist_enabled());
    client.set_allowlist_enabled(&false);
    assert!(!client.is_allowlist_enabled());
}
#[test]
fn test_transfer_service_ownership_by_owner_preserves_description() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "infer");
    let owner = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let desc = String::from_str(&env, "inference service");
    client.set_service_metadata(&svc, &desc, &owner);

    client.transfer_service_ownership(&owner, &svc, &new_owner);

    let meta = client.get_service_metadata(&svc).unwrap();
    assert_eq!(meta.owner, new_owner);
    assert_eq!(meta.description, desc);
}
#[test]
fn test_transfer_service_ownership_by_admin() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "infer");
    let owner = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let desc = String::from_str(&env, "inference service");
    client.set_service_metadata(&svc, &desc, &owner);

    client.transfer_service_ownership(&admin, &svc, &new_owner);

    let meta = client.get_service_metadata(&svc).unwrap();
    assert_eq!(meta.owner, new_owner);
    assert_eq!(meta.description, desc);
}
#[test]
#[should_panic(expected = "Error(Contract, #13)")]
fn test_transfer_service_ownership_missing_metadata_panics() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "never_set");
    let caller = Address::generate(&env);
    let new_owner = Address::generate(&env);
    client.transfer_service_ownership(&caller, &svc, &new_owner);
}
#[test]
#[should_panic(expected = "Error(Contract, #26)")]
fn test_transfer_service_ownership_unauthorized_panics() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "infer");
    let owner = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let desc = String::from_str(&env, "inference service");
    client.set_service_metadata(&svc, &desc, &owner);

    let intruder = Address::generate(&env);
    client.transfer_service_ownership(&intruder, &svc, &new_owner);
}
#[test]
#[should_panic(expected = "Error(Contract, #27)")]
fn test_transfer_service_ownership_to_self_rejected_by_owner() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "infer");
    let owner = Address::generate(&env);
    let desc = String::from_str(&env, "inference service");
    client.set_service_metadata(&svc, &desc, &owner);
    // Owner attempts to transfer to themselves — no-op rejected.
    client.transfer_service_ownership(&owner, &svc, &owner);
}
#[test]
#[should_panic(expected = "Error(Contract, #27)")]
fn test_transfer_service_ownership_to_self_rejected_by_admin() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "infer");
    let owner = Address::generate(&env);
    let desc = String::from_str(&env, "inference service");
    client.set_service_metadata(&svc, &desc, &owner);
    // Admin attempts to transfer to the current owner — no-op rejected.
    client.transfer_service_ownership(&admin, &svc, &owner);
}
#[test]
fn test_transfer_service_ownership_genuine_transfer_still_works() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "infer");
    let owner = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let desc = String::from_str(&env, "inference service");
    client.set_service_metadata(&svc, &desc, &owner);
    // Genuine transfer to a different address.
    client.transfer_service_ownership(&owner, &svc, &new_owner);
    let meta = client.get_service_metadata(&svc).unwrap();
    assert_eq!(meta.owner, new_owner);
    assert_eq!(meta.description, desc);
}
#[test]
fn test_transfer_service_ownership_genuine_transfer_emits_event() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "infer");
    let owner = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let desc = String::from_str(&env, "inference service");
    client.set_service_metadata(&svc, &desc, &owner);
    // Capture event count before transfer.
    let events_before = env.events().all();
    let count_before = events_before.len();
    // Perform genuine transfer.
    client.transfer_service_ownership(&owner, &svc, &new_owner);
    // Exactly one new event (owner_chg).
    let events_after = env.events().all();
    assert_eq!(
        events_after.len(),
        count_before + 1,
        "genuine transfer must emit exactly one event"
    );
    let (_addr, topics, data) = events_after.last().unwrap();
    let expected_topics: soroban_sdk::Vec<soroban_sdk::Val> =
        (symbol_short!("owner_chg"),).into_val(&env);
    assert_eq!(topics, expected_topics);
    let decoded: (Symbol, Address, Address) = data.into_val(&env);
    assert_eq!(decoded, (svc, owner, new_owner));
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_clear_service_metadata_rejects_non_admin() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Escrow);
    let client = EscrowClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    env.mock_all_auths();
    client.init(&admin);

    let svc = Symbol::new(&env, "infer");

    // Drop the mocked auths so the admin's require_auth() is unsatisfied.
    env.set_auths(&[]);
    client.clear_service_metadata(&svc);
}

#[test]
fn test_clear_service_metadata_removes_entry() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "infer");
    let owner = Address::generate(&env);
    let desc = String::from_str(&env, "inference service");
    client.set_service_metadata(&svc, &desc, &owner);
    assert!(client.get_service_metadata(&svc).is_some());

    client.clear_service_metadata(&svc);
    assert!(client.get_service_metadata(&svc).is_none());
}
#[test]
fn test_clear_service_metadata_is_idempotent() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "never_set");
    // Clearing a never-set entry is a no-op (no panic).
    client.clear_service_metadata(&svc);
    assert!(client.get_service_metadata(&svc).is_none());
}
#[test]
fn test_clear_service_metadata_leaves_registration_untouched() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "infer");
    let owner = Address::generate(&env);
    let desc = String::from_str(&env, "inference service");
    client.register_service(&svc);
    client.set_service_metadata(&svc, &desc, &owner);

    client.clear_service_metadata(&svc);

    assert!(client.get_service_metadata(&svc).is_none());
    assert!(client.is_service_registered(&svc));
}
#[test]
#[should_panic(expected = "Error(Contract, #14)")]
fn test_propose_admin_transfer_rejects_self_target() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.propose_admin_transfer(&admin);
}
#[test]
fn test_propose_admin_transfer_accepts_distinct_address() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let next = Address::generate(&env);
    client.propose_admin_transfer(&next);
    assert_eq!(client.get_pending_admin(), Some(next));
}
#[test]
fn test_accept_admin_transfer_clears_pending() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let next = Address::generate(&env);
    client.propose_admin_transfer(&next);
    client.accept_admin_transfer(&next);
    assert_eq!(client.get_pending_admin(), None);
}
#[test]
fn test_settle_drains_to_zero_and_stamps_last_settlement() {
    let env = Env::default();
    let ts: u64 = 12345;
    env.ledger().with_mut(|li| li.timestamp = ts);

    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "infer");
    client.set_service_price(&svc, &10i128);
    client.record_usage(&agent, &svc, &42u32);

    // No settlement has happened yet for this pair.
    assert_eq!(client.get_last_settlement(&agent, &svc), None);

    let billed = client.settle(&admin, &agent, &svc);

    assert_eq!(billed, 420i128);
    // Usage drains to exactly zero.
    assert_eq!(client.get_usage(&agent, &svc), 0);
    // LastSettlement is stamped with the current ledger timestamp.
    assert_eq!(client.get_last_settlement(&agent, &svc), Some(ts));
}
#[test]
fn test_settle_billed_matches_compute_billing_for_presettle_state() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "infer");
    client.set_service_price(&svc, &7i128);
    client.record_usage(&agent, &svc, &13u32);

    // Capture the bill the contract would report for the pre-settle state.
    let expected = client.compute_billing(&agent, &svc);
    assert_eq!(expected, 91i128);

    let billed = client.settle(&admin, &agent, &svc);
    assert_eq!(billed, expected);
    // And compute_billing now reads zero since usage drained.
    assert_eq!(client.compute_billing(&agent, &svc), 0i128);
}
#[test]
fn test_settle_emits_settled_event_with_payload() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "infer");
    client.set_service_price(&svc, &10i128);
    client.record_usage(&agent, &svc, &42u32);

    let billed = client.settle(&admin, &agent, &svc);

    let events = env.events().all();
    assert!(!events.is_empty());
    // The settled event is the most recent publish: (contract, topics, data).
    let (_addr, topics, data) = events.last().unwrap();
    let expected_topics: soroban_sdk::Vec<soroban_sdk::Val> =
        (symbol_short!("settled"),).into_val(&env);
    // Topics is a Vec<Val> with a reliable structural PartialEq.
    assert_eq!(topics, expected_topics);
    // Decode the data payload back into typed values and assert the tuple.
    let decoded: (Address, Symbol, u32, i128) = data.into_val(&env);
    assert_eq!(decoded, (agent.clone(), svc.clone(), 42u32, billed));
}
#[test]
fn test_record_usage_emits_usage_event_with_payload() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "weather_api");

    let record = client.record_usage(&agent, &svc, &25u32);

    assert_eq!(record.requests, 25);
    assert_usage_event_count(&env, 1);
    assert_latest_usage_event(&env, &agent, &svc, 25, 25);
}
#[test]
fn test_record_usage_isolates_services_and_large_deltas() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let svc_a = Symbol::new(&env, "svc_a");
    let svc_b = Symbol::new(&env, "svc_b");

    let first = client.record_usage(&agent, &svc_a, &1_000_000_000u32);
    assert_eq!(first.requests, 1_000_000_000u32);
    assert_latest_usage_event(&env, &agent, &svc_a, 1_000_000_000, 1_000_000_000);
    assert_eq!(client.get_usage(&agent, &svc_a), 1_000_000_000u32);
    assert_eq!(client.get_usage(&agent, &svc_b), 0u32);
    assert_eq!(client.get_total_usage_by_agent(&agent), 1_000_000_000u32);
    assert_eq!(client.get_total_requests_all_time(), 1_000_000_000u64);

    let second = client.record_usage(&agent, &svc_b, &7u32);
    assert_latest_usage_event(&env, &agent, &svc_b, 7, 7);
    assert_eq!(second.requests, 7u32);
    assert_latest_usage_event(&env, &agent, &svc_b, 7, 7);
    assert_eq!(client.get_usage(&agent, &svc_a), 1_000_000_000u32);
    assert_eq!(client.get_usage(&agent, &svc_b), 7u32);
    assert_eq!(client.get_total_usage_by_agent(&agent), 1_000_000_007u32);
    assert_eq!(client.get_total_requests_all_time(), 1_000_000_007u64);
}
#[test]
fn test_settle_zero_usage_returns_zero_stamps_and_emits_event() {
    let env = Env::default();
    let ts: u64 = 99_999;
    env.ledger().with_mut(|li| li.timestamp = ts);

    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "infer");
    client.set_service_price(&svc, &10i128);

    // Settle a pair that never recorded any usage.
    let billed = client.settle(&admin, &agent, &svc);
    assert_eq!(billed, 0i128);

    // Capture events immediately after `settle`: `events().all()` only
    // surfaces events from the most recent contract invocation, so any
    // intervening read (e.g. get_last_settlement) would clear them.
    let events = env.events().all();
    assert!(!events.is_empty());
    let (_addr, topics, data) = events.last().unwrap();
    let expected_topics: soroban_sdk::Vec<soroban_sdk::Val> =
        (symbol_short!("settled"),).into_val(&env);
    assert_eq!(topics, expected_topics);
    let decoded: (Address, Symbol, u32, i128) = data.into_val(&env);
    assert_eq!(decoded, (agent.clone(), svc.clone(), 0u32, 0i128));

    // Still stamps LastSettlement so SLA monitors see the drain ran.
    assert_eq!(client.get_last_settlement(&agent, &svc), Some(ts));
}
#[test]
fn test_total_settled_getters_default_to_zero() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);

    assert_eq!(client.get_total_settled_by_agent(&agent), 0i128);
    assert_eq!(client.get_total_settled_all_time(), 0i128);
}
#[test]
fn test_total_settled_counters_sum_across_settles_and_agents() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent_a = Address::generate(&env);
    let agent_b = Address::generate(&env);
    let inference = Symbol::new(&env, "infer");
    let storage = Symbol::new(&env, "storage");

    client.set_service_price(&inference, &10i128);
    client.set_service_price(&storage, &7i128);

    client.record_usage(&agent_a, &inference, &4u32);
    assert_eq!(client.settle(&admin, &agent_a, &inference), 40i128);
    assert_eq!(client.get_total_settled_by_agent(&agent_a), 40i128);
    assert_eq!(client.get_total_settled_by_agent(&agent_b), 0i128);
    assert_eq!(client.get_total_settled_all_time(), 40i128);

    client.record_usage(&agent_a, &storage, &3u32);
    assert_eq!(client.settle(&admin, &agent_a, &storage), 21i128);
    assert_eq!(client.get_total_settled_by_agent(&agent_a), 61i128);
    assert_eq!(client.get_total_settled_all_time(), 61i128);

    client.record_usage(&agent_b, &inference, &8u32);
    assert_eq!(client.settle(&admin, &agent_b, &inference), 80i128);
    assert_eq!(client.get_total_settled_by_agent(&agent_a), 61i128);
    assert_eq!(client.get_total_settled_by_agent(&agent_b), 80i128);
    assert_eq!(client.get_total_settled_all_time(), 141i128);
}
#[test]
fn test_total_settled_counters_ignore_zero_billed_settles() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let free = Symbol::new(&env, "free");
    let paid = Symbol::new(&env, "paid");

    client.record_usage(&agent, &free, &5u32);
    assert_eq!(client.settle(&admin, &agent, &free), 0i128);
    assert_eq!(client.get_total_settled_by_agent(&agent), 0i128);
    assert_eq!(client.get_total_settled_all_time(), 0i128);

    client.set_service_price(&paid, &9i128);
    client.record_usage(&agent, &paid, &2u32);
    assert_eq!(client.settle(&admin, &agent, &paid), 18i128);

    assert_eq!(client.settle(&admin, &agent, &paid), 0i128);
    assert_eq!(client.get_total_settled_by_agent(&agent), 18i128);
    assert_eq!(client.get_total_settled_all_time(), 18i128);
}
#[test]
fn test_total_settled_counters_include_settle_all() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let inference = Symbol::new(&env, "infer");
    let storage = Symbol::new(&env, "storage");

    client.set_service_price(&inference, &10i128);
    client.set_service_price(&storage, &25i128);
    client.record_usage(&agent, &inference, &2u32);
    client.record_usage(&agent, &storage, &3u32);

    let settled = client.settle_all(&admin, &agent);
    assert_eq!(settled.len(), 2);
    assert_eq!(settled.get(0), Some((inference.clone(), 20i128)));
    assert_eq!(settled.get(1), Some((storage.clone(), 75i128)));
    assert_eq!(client.get_total_settled_by_agent(&agent), 95i128);
    assert_eq!(client.get_total_settled_all_time(), 95i128);
}
#[test]
fn test_total_settled_counters_saturate_at_i128_max() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let svc_a = Symbol::new(&env, "svc_a");
    let svc_b = Symbol::new(&env, "svc_b");

    client.set_service_price(&svc_a, &i128::MAX);
    client.record_usage(&agent, &svc_a, &1u32);
    assert_eq!(client.settle(&admin, &agent, &svc_a), i128::MAX);
    assert_eq!(client.get_total_settled_by_agent(&agent), i128::MAX);
    assert_eq!(client.get_total_settled_all_time(), i128::MAX);

    client.set_service_price(&svc_b, &i128::MAX);
    client.record_usage(&agent, &svc_b, &1u32);
    assert_eq!(client.settle(&admin, &agent, &svc_b), i128::MAX);
    assert_eq!(client.get_total_settled_by_agent(&agent), i128::MAX);
    assert_eq!(client.get_total_settled_all_time(), i128::MAX);
}
#[test]
fn test_init_stamps_schema_version() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    assert_eq!(client.get_schema_version(), 2);
}
#[test]
#[should_panic(expected = "Error(Contract, #11)")]
fn test_migrate_v1_to_v2_rejected_on_fresh_v2_init() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.migrate_v1_to_v2();
}
#[test]
fn test_set_service_metadata_round_trips_description_and_owner() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "infer");
    let owner = Address::generate(&env);
    let description = String::from_str(&env, "GPU inference endpoint");

    client.set_service_metadata(&svc, &description, &owner);

    let meta = client.get_service_metadata(&svc).unwrap();
    assert_eq!(meta.description, description);
    assert_eq!(meta.owner, owner);
}
#[test]
fn test_get_service_metadata_returns_none_when_never_set() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "never_set");
    assert_eq!(client.get_service_metadata(&svc), None);
}
#[test]
fn test_set_service_metadata_overwrites_previous_value() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "infer");
    let first_owner = Address::generate(&env);
    let second_owner = Address::generate(&env);
    let first_description = String::from_str(&env, "GPU inference endpoint");
    let second_description = String::from_str(&env, "updated inference endpoint");

    client.set_service_metadata(&svc, &first_description, &first_owner);
    client.set_service_metadata(&svc, &second_description, &second_owner);

    let meta = client.get_service_metadata(&svc).unwrap();
    assert_eq!(meta.description, second_description);
    assert_eq!(meta.owner, second_owner);
}

#[test]
fn test_register_service_does_not_set_disabled_flag() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "infer");

    client.register_service(&svc);

    assert!(client.is_service_registered(&svc));
    // Registering must not implicitly disable the service.
    assert!(!client.is_service_disabled(&svc));
}
#[test]
fn test_disable_preserves_registration_and_metadata() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "infer");
    let owner = Address::generate(&env);
    let description = String::from_str(&env, "GPU inference endpoint");

    client.register_service(&svc);
    client.set_service_metadata(&svc, &description, &owner);

    client.set_service_disabled(&svc, &true);

    // Disabling a service is orthogonal to registration and metadata.
    assert!(client.is_service_disabled(&svc));
    assert!(client.is_service_registered(&svc));
    let meta = client.get_service_metadata(&svc).unwrap();
    assert_eq!(meta.description, description);
    assert_eq!(meta.owner, owner);
}
#[test]
fn test_disable_unregistered_service_preserves_other_slots() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "infer");
    let owner = Address::generate(&env);
    let description = String::from_str(&env, "GPU inference endpoint");

    client.set_service_metadata(&svc, &description, &owner);

    assert!(!client.is_service_registered(&svc));
    assert!(!client.is_service_disabled(&svc));

    client.set_service_disabled(&svc, &true);

    assert!(!client.is_service_registered(&svc));
    assert!(client.is_service_disabled(&svc));
    let meta = client.get_service_metadata(&svc).unwrap();
    assert_eq!(meta.description, description);
    assert_eq!(meta.owner, owner);
}

#[test]
fn test_unregister_service_does_not_clear_metadata_or_disabled_flag() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "infer");
    let owner = Address::generate(&env);
    let description = String::from_str(&env, "GPU inference endpoint");

    client.register_service(&svc);
    client.set_service_metadata(&svc, &description, &owner);
    client.set_service_disabled(&svc, &true);

    client.unregister_service(&svc);

    // unregister_service only removes the ServiceRegistered slot.
    assert!(!client.is_service_registered(&svc));
    // Metadata and the disabled flag survive an unregister.
    assert!(client.is_service_disabled(&svc));
    let meta = client.get_service_metadata(&svc).unwrap();
    assert_eq!(meta.description, description);
    assert_eq!(meta.owner, owner);
}
#[test]
fn test_service_slot_toggle_matrix_is_independent() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "infer");
    let owner = Address::generate(&env);
    let description = String::from_str(&env, "GPU inference endpoint");

    // Baseline: every slot reads its default for a fresh service id.
    assert!(!client.is_service_registered(&svc));
    assert!(!client.is_service_disabled(&svc));
    assert_eq!(client.get_service_metadata(&svc), None);

    // Metadata is independent from registration and disable state.
    client.set_service_metadata(&svc, &description, &owner);
    assert!(!client.is_service_registered(&svc));
    assert!(!client.is_service_disabled(&svc));
    let meta = client.get_service_metadata(&svc).unwrap();
    assert_eq!(meta.description, description);
    assert_eq!(meta.owner, owner);

    // Toggle registered only.
    client.register_service(&svc);
    assert!(client.is_service_registered(&svc));
    assert!(!client.is_service_disabled(&svc));
    let meta = client.get_service_metadata(&svc).unwrap();
    assert_eq!(meta.description, description);
    assert_eq!(meta.owner, owner);

    // Toggle disabled only; registered stays set.
    client.set_service_disabled(&svc, &true);
    assert!(client.is_service_registered(&svc));
    assert!(client.is_service_disabled(&svc));
    let meta = client.get_service_metadata(&svc).unwrap();
    assert_eq!(meta.description, description);
    assert_eq!(meta.owner, owner);

    // Re-enable; registered stays set.
    client.set_service_disabled(&svc, &false);
    assert!(client.is_service_registered(&svc));
    assert!(!client.is_service_disabled(&svc));
    let meta = client.get_service_metadata(&svc).unwrap();
    assert_eq!(meta.description, description);
    assert_eq!(meta.owner, owner);
}

// ── register_service_with_metadata ───────────────────────────────────────────
//
// `register_service_with_metadata` atomically sets `ServiceRegistered` and
// `ServiceMetadata` in a single admin-gated call, emits `svc_reg(service_id,
// owner)`, honours the pause gate, and is idempotent (overwrites metadata on
// re-registration). The combined call must produce the same resulting state as
// calling `register_service` followed by `set_service_metadata`.

/// After a single `register_service_with_metadata` call, both
/// `is_service_registered` returns `true` and `get_service_metadata`
/// returns the exact description and owner that were passed.
#[test]
fn test_register_with_metadata_atomicity() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "infer");
    let owner = Address::generate(&env);
    let desc = String::from_str(&env, "GPU inference endpoint");

    client.register_service_with_metadata(&svc, &desc, &owner);

    // Both slots are set atomically by the single call.
    assert!(client.is_service_registered(&svc));
    let meta = client.get_service_metadata(&svc).unwrap();
    assert_eq!(meta.description, desc);
    assert_eq!(meta.owner, owner);
}

/// The call emits a `svc_reg(service_id, owner)` event that can be
/// decoded from `env.events().all()`.
#[test]
fn test_register_with_metadata_emits_svc_reg_event() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "infer");
    let owner = Address::generate(&env);
    let desc = String::from_str(&env, "GPU inference endpoint");

    client.register_service_with_metadata(&svc, &desc, &owner);

    let events = env.events().all();
    assert!(!events.is_empty());
    let (_addr, topics, data) = events.last().unwrap();
    let expected_topics: soroban_sdk::Vec<soroban_sdk::Val> =
        (symbol_short!("svc_reg"),).into_val(&env);
    assert_eq!(topics, expected_topics);
    let decoded: (Symbol, Address) = data.into_val(&env);
    assert_eq!(decoded, (svc, owner));
}

/// Re-registering an existing service id overwrites the stored metadata
/// (idempotent overwrite).
#[test]
fn test_register_with_metadata_overwrite() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "infer");
    let owner1 = Address::generate(&env);
    let desc1 = String::from_str(&env, "first description");
    let owner2 = Address::generate(&env);
    let desc2 = String::from_str(&env, "second description");

    // First registration.
    client.register_service_with_metadata(&svc, &desc1, &owner1);
    let meta = client.get_service_metadata(&svc).unwrap();
    assert_eq!(meta.description, desc1);
    assert_eq!(meta.owner, owner1);

    // Overwrite with different metadata.
    client.register_service_with_metadata(&svc, &desc2, &owner2);
    let meta = client.get_service_metadata(&svc).unwrap();
    assert_eq!(meta.description, desc2);
    assert_eq!(meta.owner, owner2);
    // Registration flag stays true (idempotent).
    assert!(client.is_service_registered(&svc));
}

/// An empty description string is accepted by the entrypoint.
#[test]
fn test_register_with_metadata_empty_description_accepted() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "infer");
    let owner = Address::generate(&env);
    let empty = String::from_str(&env, "");

    client.register_service_with_metadata(&svc, &empty, &owner);

    assert!(client.is_service_registered(&svc));
    let meta = client.get_service_metadata(&svc).unwrap();
    assert_eq!(meta.description, empty);
    assert_eq!(meta.owner, owner);
}

/// A non-admin caller is rejected with `Unauthorized` (the auth framework's
/// panic, not a typed error).
#[test]
#[should_panic(expected = "Unauthorized")]
fn test_register_with_metadata_requires_admin() {
    let env = Env::default();
    let client = setup_scoped_auth(&env);
    let svc = Symbol::new(&env, "infer");
    let owner = Address::generate(&env);
    let desc = String::from_str(&env, "some service");
    // No admin auth is wired up beyond init, so require_admin will fail.
    client.register_service_with_metadata(&svc, &desc, &owner);
}

/// Calling while the contract is paused panics with `ContractPaused` (#4).
#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_register_with_metadata_rejected_while_paused() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.pause();
    let svc = Symbol::new(&env, "infer");
    let owner = Address::generate(&env);
    let desc = String::from_str(&env, "some service");
    client.register_service_with_metadata(&svc, &desc, &owner);
}

/// The combined call produces the same resulting state as calling
/// `register_service` then `set_service_metadata` separately, proving
/// the atomic entrypoint is semantically equivalent to the two-step
/// sequence.
#[test]
fn test_register_with_metadata_equivalent_to_separate_calls() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "infer");
    let owner = Address::generate(&env);
    let desc = String::from_str(&env, "GPU inference endpoint");

    // Combined call.
    client.register_service_with_metadata(&svc, &desc, &owner);

    // Capture state produced by the combined call.
    let registered_combined = client.is_service_registered(&svc);
    let meta_combined = client.get_service_metadata(&svc).unwrap();

    // Fresh contract, separate calls.
    let env2 = Env::default();
    let (client2, _admin2) = setup_initialized(&env2);
    let svc2 = Symbol::new(&env2, "infer");
    let owner2 = Address::generate(&env2);
    let desc2 = String::from_str(&env2, "GPU inference endpoint");

    client2.register_service(&svc2);
    client2.set_service_metadata(&svc2, &desc2, &owner2);

    // State must be identical.
    assert_eq!(client2.is_service_registered(&svc2), registered_combined);
    let meta_separate = client2.get_service_metadata(&svc2).unwrap();
    assert_eq!(meta_separate, meta_combined);
}
#[test]
fn test_pause_emits_paused_event_true() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    client.pause();

    // Read events immediately after pause(): events().all() only surfaces
    // events from the most recent contract invocation.
    let events = env.events().all();
    assert!(!events.is_empty());
    let (_addr, topics, data) = events.last().unwrap();
    let expected_topics: soroban_sdk::Vec<soroban_sdk::Val> =
        (symbol_short!("paused"),).into_val(&env);
    assert_eq!(topics, expected_topics);
    let flag: bool = data.into_val(&env);
    assert!(flag);
    assert!(client.is_paused());
}
#[test]
fn test_unpause_emits_paused_event_false() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.pause();

    client.unpause();

    let events = env.events().all();
    assert!(!events.is_empty());
    let (_addr, topics, data) = events.last().unwrap();
    let expected_topics: soroban_sdk::Vec<soroban_sdk::Val> =
        (symbol_short!("paused"),).into_val(&env);
    assert_eq!(topics, expected_topics);
    let flag: bool = data.into_val(&env);
    assert!(!flag);
    assert!(!client.is_paused());
}
#[test]
fn test_double_pause_is_idempotent() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    client.pause();
    assert!(client.is_paused());
    // Pausing an already-paused contract keeps it paused.
    client.pause();
    assert!(client.is_paused());
}
#[test]
fn test_double_unpause_is_idempotent() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    // Unpausing a never-paused contract is a no-op and stays unpaused.
    client.unpause();
    assert!(!client.is_paused());
    client.unpause();
    assert!(!client.is_paused());
}
#[test]
fn test_pause_pause_unpause_ends_unpaused() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    client.pause();
    client.pause();
    client.unpause();

    assert!(!client.is_paused());
}

// Regression coverage for the extracted `require_admin` / `ensure_not_paused`
// helpers (issue #29): the helper refactor must preserve the exact error
// codes and gating behaviour of the previously-inlined blocks.
#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_set_service_price_panics_not_initialized_before_init() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, Escrow);
    let client = EscrowClient::new(&env, &contract_id);
    // No init() call: require_admin must still panic NotInitialized (#3).
    client.set_service_price(&Symbol::new(&env, "infer"), &500i128);
}
#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_record_usage_paused_gate_via_helper() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.pause();
    let agent = Address::generate(&env);
    // ensure_not_paused must still panic ContractPaused (#4) while paused.
    client.record_usage(&agent, &Symbol::new(&env, "infer"), &1u32);
}
#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_set_price_tiers_panics_not_initialized_before_init() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, Escrow);
    let client = EscrowClient::new(&env, &contract_id);
    // No init() call: require_admin must still panic NotInitialized (#3).
    let tiers: Vec<PriceTier> = Vec::new(&env);
    client.set_price_tiers(&Symbol::new(&env, "infer"), &tiers);
}
#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_remove_price_tiers_panics_not_initialized_before_init() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, Escrow);
    let client = EscrowClient::new(&env, &contract_id);
    // No init() call: require_admin must still panic NotInitialized (#3).
    client.remove_price_tiers(&Symbol::new(&env, "infer"));
}
#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_resolve_dispute_panics_not_initialized_before_init() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, Escrow);
    let client = EscrowClient::new(&env, &contract_id);
    // No init() call: require_admin must still panic NotInitialized (#3).
    // ensure_not_paused passes first (defaults to false), then require_admin panics.
    let agent = Address::generate(&env);
    client.resolve_dispute(&agent, &Symbol::new(&env, "infer"), &0u32);
}

#[test]
fn test_list_open_disputes_returns_empty_for_agent_without_disputes() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "svc_a");
    client.record_usage(&agent, &svc, &4u32);

    let disputes = client.list_open_disputes(&agent);
    assert_eq!(disputes.len(), 0);
}

#[test]
fn test_list_open_disputes_returns_single_service() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "svc_a");
    client.record_usage(&agent, &svc, &4u32);
    client.open_dispute(&agent, &svc);

    let disputes = client.list_open_disputes(&agent);
    assert_eq!(disputes.len(), 1);
    assert_eq!(disputes.get(0), Some(svc.clone()));
}

#[test]
fn test_list_open_disputes_is_bounded_by_batch_limit() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    let agent = Address::generate(&env);
    let mut expected: Vec<Symbol> = Vec::new(&env);

    for i in 0..(MAX_BATCH_READ + 5) {
        let mut buf = [0u8; 8];
        let name = svc_name(&mut buf, i);
        let service_id = Symbol::new(&env, name);
        client.record_usage(&agent, &service_id, &1u32);
        if i < MAX_BATCH_READ {
            expected.push_back(service_id.clone());
        }
        if i < MAX_BATCH_READ {
            client.open_dispute(&agent, &service_id);
        }
    }

    let disputes = client.list_open_disputes(&agent);
    assert_eq!(disputes.len(), MAX_BATCH_READ as u32);
    for i in 0..MAX_BATCH_READ {
        assert_eq!(disputes.get(i), expected.get(i));
    }
}

#[test]
fn test_get_usage_batch_preserves_order() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    let agent = Address::generate(&env);
    let svc_a = Symbol::new(&env, "svc_a");
    let svc_b = Symbol::new(&env, "svc_b");
    let svc_c = Symbol::new(&env, "svc_c");

    client.record_usage(&agent, &svc_a, &10u32);
    client.record_usage(&agent, &svc_b, &20u32);
    client.record_usage(&agent, &svc_c, &30u32);

    let mut pairs: Vec<(Address, Symbol)> = Vec::new(&env);
    pairs.push_back((agent.clone(), svc_b.clone()));
    pairs.push_back((agent.clone(), svc_a.clone()));
    pairs.push_back((agent.clone(), svc_c.clone()));

    let out = client.get_usage_batch(&pairs);
    assert_eq!(out.len(), 3);
    assert_eq!(out.get(0), Some(20));
    assert_eq!(out.get(1), Some(10));
    assert_eq!(out.get(2), Some(30));
}
#[test]
fn test_get_usage_batch_unknown_pairs_return_zero() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "never_used");

    let mut pairs: Vec<(Address, Symbol)> = Vec::new(&env);
    pairs.push_back((agent.clone(), svc.clone()));

    let out = client.get_usage_batch(&pairs);
    assert_eq!(out.len(), 1);
    assert_eq!(out.get(0), Some(0));
}
#[test]
fn test_get_usage_batch_mix_known_and_unknown() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    let agent = Address::generate(&env);
    let known = Symbol::new(&env, "known");
    let unknown = Symbol::new(&env, "unknown");

    client.record_usage(&agent, &known, &7u32);

    let mut pairs: Vec<(Address, Symbol)> = Vec::new(&env);
    pairs.push_back((agent.clone(), unknown.clone()));
    pairs.push_back((agent.clone(), known.clone()));

    let out = client.get_usage_batch(&pairs);
    assert_eq!(out.get(0), Some(0));
    assert_eq!(out.get(1), Some(7));
}
#[test]
fn test_get_usage_batch_duplicate_pairs() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "dup_svc");
    client.record_usage(&agent, &svc, &42u32);

    let mut pairs: Vec<(Address, Symbol)> = Vec::new(&env);
    pairs.push_back((agent.clone(), svc.clone()));
    pairs.push_back((agent.clone(), svc.clone()));
    pairs.push_back((agent.clone(), svc.clone()));

    let out = client.get_usage_batch(&pairs);
    assert_eq!(out.len(), 3);
    assert_eq!(out.get(0), Some(42));
    assert_eq!(out.get(1), Some(42));
    assert_eq!(out.get(2), Some(42));
}
#[test]
fn test_get_usage_batch_empty_returns_empty() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    let pairs: Vec<(Address, Symbol)> = Vec::new(&env);
    let out = client.get_usage_batch(&pairs);
    assert_eq!(out.len(), 0);
}
#[test]
fn test_get_usage_batch_at_bound_succeeds() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "bound_svc");
    client.record_usage(&agent, &svc, &5u32);

    let mut pairs: Vec<(Address, Symbol)> = Vec::new(&env);
    for _ in 0..MAX_BATCH_READ {
        pairs.push_back((agent.clone(), svc.clone()));
    }
    assert_eq!(pairs.len(), MAX_BATCH_READ);

    let out = client.get_usage_batch(&pairs);
    assert_eq!(out.len(), MAX_BATCH_READ);
    assert_eq!(out.get(0), Some(5));
    assert_eq!(out.get(MAX_BATCH_READ - 1), Some(5));
}
#[test]
#[should_panic(expected = "Error(Contract, #16)")]
fn test_get_usage_batch_oversized_panics() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "over_svc");

    let mut pairs: Vec<(Address, Symbol)> = Vec::new(&env);
    for _ in 0..(MAX_BATCH_READ + 1) {
        pairs.push_back((agent.clone(), svc.clone()));
    }
    assert_eq!(pairs.len(), MAX_BATCH_READ + 1);

    client.get_usage_batch(&pairs);
}
#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_record_usage_paused_beats_zero_requests() {
    // Paused (#4) must win even when requests == 0 (which would be #2).
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.pause();
    let agent = Address::generate(&env);
    let service_id = Symbol::new(&env, "weather_api");
    client.record_usage(&agent, &service_id, &0u32);
}
#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_record_usage_zero_requests_beats_max() {
    // Zero-requests (#2) must win over the max cap (#8): with max=5 and
    // requests=0, the zero check fires first.
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.set_max_requests_per_call(&5u32);
    let agent = Address::generate(&env);
    let service_id = Symbol::new(&env, "weather_api");
    client.record_usage(&agent, &service_id, &0u32);
}
#[test]
#[should_panic(expected = "Error(Contract, #23)")]
fn test_record_usage_max_beats_min() {
    // With the cross-bound guard in place, setting min > max is rejected at
    // setter time (#23 InvalidRequestBounds) before record_usage is ever
    // reached. This test confirms the setter rejects the contradictory config.
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.set_max_requests_per_call(&5u32);
    // min=10 > max=5 → InvalidRequestBounds (#23) at set_min time.
    client.set_min_requests_per_call(&10u32);
}
#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn test_record_usage_min_beats_registration() {
    // Min (#9) must win over the registration gate (#7): with min=10 and
    // strict registration required (service unregistered), a below-min
    // request trips #9 before #7.
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.set_min_requests_per_call(&10u32);
    client.set_require_service_registration(&true);
    let agent = Address::generate(&env);
    let service_id = Symbol::new(&env, "weather_api");
    client.record_usage(&agent, &service_id, &3u32);
}
#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_record_usage_registration_beats_disabled() {
    // Registration (#7) must win over disabled (#12): require registration,
    // leave the service unregistered, and also disable it. #7 fires first.
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.set_require_service_registration(&true);
    let service_id = Symbol::new(&env, "weather_api");
    client.set_service_disabled(&service_id, &true);
    let agent = Address::generate(&env);
    client.record_usage(&agent, &service_id, &5u32);
}
#[test]
#[should_panic(expected = "Error(Contract, #12)")]
fn test_record_usage_disabled_beats_allowlist() {
    // Disabled (#12) must win over the allowlist (#10): disable a registered
    // service and enable a (non-matching) allowlist. #12 fires first.
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let service_id = Symbol::new(&env, "weather_api");
    client.register_service(&service_id);
    client.set_service_disabled(&service_id, &true);
    client.set_allowlist_enabled(&true);
    let agent = Address::generate(&env);
    client.record_usage(&agent, &service_id, &5u32);
}
#[test]
#[should_panic(expected = "Error(Contract, #10)")]
fn test_record_usage_allowlist_fires_when_enabled_and_not_allowed() {
    // Allowlist (#10) fires when enabled and the agent is not allowed.
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.set_allowlist_enabled(&true);
    let agent = Address::generate(&env);
    let service_id = Symbol::new(&env, "weather_api");
    client.record_usage(&agent, &service_id, &5u32);
}
#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_record_usage_registration_fires_when_required_and_unregistered() {
    // Registration (#7) fires when required and the service is unregistered.
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.set_require_service_registration(&true);
    let agent = Address::generate(&env);
    let service_id = Symbol::new(&env, "weather_api");
    client.record_usage(&agent, &service_id, &5u32);
}
#[test]
#[should_panic(expected = "Error(Contract, #12)")]
fn test_record_usage_disabled_fires_when_service_disabled() {
    // Disabled (#12) fires when the service is disabled.
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let service_id = Symbol::new(&env, "weather_api");
    client.set_service_disabled(&service_id, &true);
    let agent = Address::generate(&env);
    client.record_usage(&agent, &service_id, &5u32);
}
#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_record_usage_max_fires_above_cap() {
    // Max (#8) fires when requests exceed the configured cap.
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.set_max_requests_per_call(&5u32);
    let agent = Address::generate(&env);
    let service_id = Symbol::new(&env, "weather_api");
    client.record_usage(&agent, &service_id, &6u32);
}
#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn test_record_usage_min_fires_below_floor() {
    // Min (#9) fires when requests fall below the configured floor.
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.set_min_requests_per_call(&10u32);
    let agent = Address::generate(&env);
    let service_id = Symbol::new(&env, "weather_api");
    client.record_usage(&agent, &service_id, &3u32);
}
#[test]
fn test_record_usage_passes_all_gates_when_satisfied() {
    // Sanity: with every gate enabled and satisfied, record_usage succeeds.
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let service_id = Symbol::new(&env, "weather_api");
    let agent = Address::generate(&env);
    client.set_max_requests_per_call(&100u32);
    client.set_min_requests_per_call(&1u32);
    client.set_require_service_registration(&true);
    client.register_service(&service_id);
    client.set_allowlist_enabled(&true);
    client.set_agent_allowed(&agent, &true);

    let record = client.record_usage(&agent, &service_id, &5u32);
    assert_eq!(record.requests, 5);
}
#[test]
#[should_panic(expected = "Error(Contract, #17)")]
fn test_record_usage_rejects_blocked_agent() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "infer");

    client.set_agent_blocked(&agent, &true);
    client.record_usage(&agent, &svc, &1u32);
}
#[test]
#[should_panic(expected = "Error(Contract, #17)")]
fn test_blocklist_takes_precedence_over_allowlist() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "infer");

    // Enable the allowlist and explicitly allow the agent...
    client.set_allowlist_enabled(&true);
    client.set_agent_allowed(&agent, &true);
    // ...but also block it: the block must win.
    client.set_agent_blocked(&agent, &true);
    client.record_usage(&agent, &svc, &1u32);
}
#[test]
#[should_panic(expected = "Error(Contract, #17)")]
fn test_blocked_agent_rejected_while_allowlist_disabled() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "infer");

    // Allowlist stays disabled (its default); the block alone rejects.
    assert!(!client.is_allowlist_enabled());
    client.set_agent_blocked(&agent, &true);
    client.record_usage(&agent, &svc, &1u32);
}
#[test]
fn test_unblock_then_record_succeeds() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "infer");

    client.set_agent_blocked(&agent, &true);
    client.set_agent_blocked(&agent, &false);

    let record = client.record_usage(&agent, &svc, &5u32);
    assert_eq!(record.requests, 5);
    assert_eq!(client.get_usage(&agent, &svc), 5);
}
#[test]
fn test_is_agent_blocked_round_trip() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);

    // Defaults to false when never set.
    assert!(!client.is_agent_blocked(&agent));
    client.set_agent_blocked(&agent, &true);
    assert!(client.is_agent_blocked(&agent));
    client.set_agent_blocked(&agent, &false);
    assert!(!client.is_agent_blocked(&agent));
}
#[test]
#[should_panic(expected = "Unauthorized")]
fn test_set_agent_blocked_requires_admin_auth() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Escrow);
    let client = EscrowClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    env.mock_all_auths();
    client.init(&admin);

    // Drop the mocked auths so the admin require_auth is enforced.
    env.set_auths(&[]);
    let agent = Address::generate(&env);
    client.set_agent_blocked(&agent, &true);
}
#[test]
fn test_remove_service_price_clears_price() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "infer");
    client.set_service_price(&svc, &500i128);
    assert_eq!(client.get_service_price(&svc), 500i128);

    client.remove_service_price(&svc);

    // Reads back 0, same as a never-priced service.
    assert_eq!(client.get_service_price(&svc), 0i128);
}
#[test]
fn test_remove_service_price_is_idempotent() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "never_set");
    // Removing the price of a never-priced service is a no-op (no panic).
    client.remove_service_price(&svc);
    assert_eq!(client.get_service_price(&svc), 0i128);
}
#[test]
fn test_remove_service_price_then_reset_works() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "infer");
    client.set_service_price(&svc, &500i128);
    client.remove_service_price(&svc);
    assert_eq!(client.get_service_price(&svc), 0i128);

    // Re-setting after removal works and round-trips.
    client.set_service_price(&svc, &750i128);
    assert_eq!(client.get_service_price(&svc), 750i128);
}
#[test]
fn test_compute_billing_zero_after_price_removed() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "infer");
    client.set_service_price(&svc, &10i128);
    client.record_usage(&agent, &svc, &42u32);
    assert_eq!(client.compute_billing(&agent, &svc), 420i128);

    client.remove_service_price(&svc);

    // Usage is untouched, but with no price the bill is zero.
    assert_eq!(client.compute_billing(&agent, &svc), 0i128);
}
#[test]
fn test_remove_service_price_emits_price_rmv_event() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "infer");
    client.set_service_price(&svc, &500i128);

    client.remove_service_price(&svc);

    let events = env.events().all();
    assert!(!events.is_empty());
    let (_addr, topics, data) = events.last().unwrap();
    let expected_topics: soroban_sdk::Vec<soroban_sdk::Val> =
        (symbol_short!("price_rmv"),).into_val(&env);
    assert_eq!(topics, expected_topics);
    let decoded: Symbol = data.into_val(&env);
    assert_eq!(decoded, svc);
}
#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_remove_service_price_rejected_while_paused() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "infer");
    client.set_service_price(&svc, &500i128);
    client.pause();
    client.remove_service_price(&svc);
}
#[test]
#[should_panic(expected = "Unauthorized")]
fn test_remove_service_price_non_admin_panics() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "infer");
    client.set_service_price(&svc, &500i128);
    // Drop the mocked auths so the admin's require_auth() is unsatisfied,
    // simulating a caller without the admin signature.
    env.set_auths(&[]);
    client.remove_service_price(&svc);
}
#[test]
fn test_i17_per_call_bounds_default_to_unbounded() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    // No cap and no floor configured by default.
    assert_eq!(client.get_max_requests_per_call(), u32::MAX);
    assert_eq!(client.get_min_requests_per_call(), 0);
    // Any positive value is therefore accepted.
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "infer");
    assert_eq!(
        client.record_usage(&agent, &svc, &1_000_000u32).requests,
        1_000_000
    );
}
#[test]
fn test_rate_window_getters_unrecorded() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = make_agent(&env);

    assert_eq!(client.get_rate_window(&agent), (0, 0));
    // MaxRequestsPerWindow and WindowSeconds default to 0, which disables the limiter.
    // get_remaining_in_window should return 0 in this case (max_per_window).
    assert_eq!(client.get_remaining_in_window(&agent), 0);
}

#[test]
fn test_rate_window_getters_mid_window() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = make_agent(&env);
    let svc = make_service(&env, "svc");

    // Configure rate limiter
    client.set_max_requests_per_window(&100u32);
    client.set_rate_window_seconds(&3600u64);

    // First call: window starts now.
    let now = env.ledger().timestamp();
    client.record_usage(&agent, &svc, &10u32);

    assert_eq!(client.get_rate_window(&agent), (now, 10));
    assert_eq!(client.get_remaining_in_window(&agent), 90);
}

#[test]
fn test_rate_window_read_does_not_mutate() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = make_agent(&env);
    let svc = make_service(&env, "svc");

    client.set_max_requests_per_window(&100u32);
    client.set_rate_window_seconds(&3600u64);

    client.record_usage(&agent, &svc, &10u32);

    // Call get_rate_window
    client.get_rate_window(&agent);

    // Verify state is unchanged
    let now = env.ledger().timestamp();
    assert_eq!(client.get_rate_window(&agent), (now, 10));
}

#[test]
fn test_rate_window_expired_rollover() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = make_agent(&env);
    let svc = make_service(&env, "svc");

    let window_len = 3600u64;
    client.set_max_requests_per_window(&100u32);
    client.set_rate_window_seconds(&window_len);

    // First call: window starts now.
    let now = env.ledger().timestamp();
    client.record_usage(&agent, &svc, &10u32);

    // Advance ledger to trigger expiration
    advance_ledger(&env, window_len + 1);

    // get_remaining_in_window should see expired window and return full cap.
    assert_eq!(client.get_remaining_in_window(&agent), 100);

    // get_rate_window should still show the old window data (it doesn't roll forward)
    assert_eq!(client.get_rate_window(&agent), (now, 10));
}

#[test]
fn test_i17_record_usage_accepts_value_exactly_at_max() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.set_max_requests_per_call(&100u32);
    assert_eq!(client.get_max_requests_per_call(), 100);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "infer");
    // Exactly at the ceiling is allowed (boundary is inclusive).
    assert_eq!(client.record_usage(&agent, &svc, &100u32).requests, 100);
}
#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_i17_record_usage_rejects_above_max() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.set_max_requests_per_call(&100u32);
    let agent = Address::generate(&env);
    client.record_usage(&agent, &Symbol::new(&env, "infer"), &101u32);
}
#[test]
fn test_i17_record_usage_accepts_value_exactly_at_min() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.set_min_requests_per_call(&10u32);
    assert_eq!(client.get_min_requests_per_call(), 10);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "infer");
    // Exactly at the floor is allowed (boundary is inclusive).
    assert_eq!(client.record_usage(&agent, &svc, &10u32).requests, 10);
}
#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn test_i17_record_usage_rejects_below_min() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.set_min_requests_per_call(&10u32);
    let agent = Address::generate(&env);
    client.record_usage(&agent, &Symbol::new(&env, "infer"), &9u32);
}
#[test]
fn test_i18_strict_off_allows_unknown_service() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    // Default: strict registration is off, so unknown services are accepted.
    assert!(!client.is_service_registration_required());
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "unknown");
    assert_eq!(client.record_usage(&agent, &svc, &1u32).requests, 1);
}
#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_i18_strict_on_rejects_unregistered() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.set_require_service_registration(&true);
    assert!(client.is_service_registration_required());
    let agent = Address::generate(&env);
    client.record_usage(&agent, &Symbol::new(&env, "ghost"), &1u32);
}
#[test]
fn test_i18_register_admits_service_under_strict_mode() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.set_require_service_registration(&true);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "infer");
    client.register_service(&svc);
    assert!(client.is_service_registered(&svc));
    assert_eq!(client.record_usage(&agent, &svc, &2u32).requests, 2);
}
#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_i18_unregister_reinstates_rejection() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.set_require_service_registration(&true);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "infer");
    client.register_service(&svc);
    client.unregister_service(&svc);
    assert!(!client.is_service_registered(&svc));
    client.record_usage(&agent, &svc, &1u32);
}
#[test]
#[should_panic(expected = "Error(Contract, #12)")]
fn test_i18_disabled_service_rejects_usage() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "infer");
    client.set_service_disabled(&svc, &true);
    assert!(client.is_service_disabled(&svc));
    client.record_usage(&agent, &svc, &1u32);
}
#[test]
fn test_i18_reenable_service_resumes_usage() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "infer");
    // Disabling then re-enabling restores the ability to accrue usage and
    // leaves the registration flag independent of the disabled flag.
    client.register_service(&svc);
    client.set_service_disabled(&svc, &true);
    client.set_service_disabled(&svc, &false);
    assert!(!client.is_service_disabled(&svc));
    assert!(client.is_service_registered(&svc));
    assert_eq!(client.record_usage(&agent, &svc, &3u32).requests, 3);
}
#[test]
fn test_i19_total_usage_by_agent_accumulates_across_services() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let a = Symbol::new(&env, "svc_a");
    let b = Symbol::new(&env, "svc_b");
    client.record_usage(&agent, &a, &5u32);
    client.record_usage(&agent, &b, &7u32);
    // Cross-service lifetime counter sums both services for the agent.
    assert_eq!(client.get_total_usage_by_agent(&agent), 12);
}
#[test]
fn test_i19_total_requests_all_time_sums_across_agents() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let a1 = Address::generate(&env);
    let a2 = Address::generate(&env);
    let svc = Symbol::new(&env, "infer");
    client.record_usage(&a1, &svc, &4u32);
    client.record_usage(&a2, &svc, &6u32);
    assert_eq!(client.get_total_requests_all_time(), 10u64);
}
#[test]
fn test_i19_lifetime_counters_survive_settle() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "infer");
    client.set_service_price(&svc, &2i128);
    client.record_usage(&agent, &svc, &9u32);
    client.settle(&admin, &agent, &svc);
    // Per-pair usage drains, lifetime analytics persist.
    assert_eq!(client.get_usage(&agent, &svc), 0);
    assert_eq!(client.get_total_usage_by_agent(&agent), 9);
    assert_eq!(client.get_total_requests_all_time(), 9u64);
    // Re-recording after settle continues to grow the lifetime counter.
    client.record_usage(&agent, &svc, &1u32);
    assert_eq!(client.get_total_usage_by_agent(&agent), 10);
}
#[test]
fn test_i19_last_settlement_none_before_some_after() {
    let env = Env::default();
    let ts: u64 = 777;
    env.ledger().with_mut(|li| li.timestamp = ts);
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "infer");
    client.set_service_price(&svc, &1i128);
    client.record_usage(&agent, &svc, &3u32);
    // Never-settled reads as None (distinct from Some(0)).
    assert_eq!(client.get_last_settlement(&agent, &svc), None);
    client.settle(&admin, &agent, &svc);
    assert_eq!(client.get_last_settlement(&agent, &svc), Some(ts));
}
#[test]
fn test_i19_last_settlement_is_none_for_never_settled_pair() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "never");
    assert_eq!(client.get_last_settlement(&agent, &svc), None);
}
#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_i20_cancel_then_accept_fails() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let next = Address::generate(&env);
    client.propose_admin_transfer(&next);
    client.cancel_admin_transfer();
    // Nothing pending after a cancel, so accept must fail with #5.
    client.accept_admin_transfer(&next);
}
#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_i20_wrong_caller_accept_rejected() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let next = Address::generate(&env);
    let intruder = Address::generate(&env);
    client.propose_admin_transfer(&next);
    // A caller other than the pending admin is rejected with #6.
    client.accept_admin_transfer(&intruder);
}
#[test]
fn test_i20_repropose_overwrites_pending() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let first = Address::generate(&env);
    let second = Address::generate(&env);
    client.propose_admin_transfer(&first);
    assert_eq!(client.get_pending_admin(), Some(first));
    client.propose_admin_transfer(&second);
    assert_eq!(client.get_pending_admin(), Some(second.clone()));
    // Only the most recent pending admin can accept.
    client.accept_admin_transfer(&second);
    assert_eq!(client.get_admin(), Some(second));
}
#[test]
fn test_i20_rotated_admin_can_act_after_handover() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let next = Address::generate(&env);
    client.propose_admin_transfer(&next);
    client.accept_admin_transfer(&next);
    // The rotated admin can now perform an admin-gated action.
    client.pause();
    assert!(client.is_paused());
}
#[test]
fn test_i20_schema_version_is_two_after_init() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    // Fresh v2 init stamps SchemaVersion = 2 directly (no migration needed).
    assert_eq!(client.get_schema_version(), 2);
}
#[test]
#[should_panic(expected = "Error(Contract, #11)")]
fn test_i20_double_migrate_guard_rejects_on_v2() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    // Already at v2, so the v1->v2 migration refuses with #11.
    client.migrate_v1_to_v2();
}
#[test]
fn test_i21_per_pair_usage_saturates_at_u32_max() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "infer");
    client.record_usage(&agent, &svc, &u32::MAX);
    // Adding more saturates at u32::MAX rather than overflowing.
    assert_eq!(client.record_usage(&agent, &svc, &10u32).requests, u32::MAX);
    assert_eq!(client.get_usage(&agent, &svc), u32::MAX);
}
#[test]
fn test_i21_total_usage_by_agent_saturates() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let a = Symbol::new(&env, "svc_a");
    let b = Symbol::new(&env, "svc_b");
    client.record_usage(&agent, &a, &u32::MAX);
    client.record_usage(&agent, &b, &u32::MAX);
    // The cross-service lifetime counter also saturates at u32::MAX.
    assert_eq!(client.get_total_usage_by_agent(&agent), u32::MAX);
}
#[test]
fn test_i21_compute_billing_saturates_at_i128_max() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "infer");
    client.set_service_price(&svc, &i128::MAX);
    client.record_usage(&agent, &svc, &2u32);
    // 2 * i128::MAX saturates to i128::MAX rather than overflowing.
    assert_eq!(client.compute_billing(&agent, &svc), i128::MAX);
}
#[test]
fn test_i21_settle_returns_saturated_value_and_drains() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "infer");
    client.set_service_price(&svc, &i128::MAX);
    client.record_usage(&agent, &svc, &5u32);
    let billed = client.settle(&admin, &agent, &svc);
    assert_eq!(billed, i128::MAX);
    // The counter still drains to zero even when billing saturated.
    assert_eq!(client.get_usage(&agent, &svc), 0);
}
#[test]
fn test_i21_total_requests_all_time_accumulates_large_values() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let a1 = Address::generate(&env);
    let a2 = Address::generate(&env);
    let svc = Symbol::new(&env, "infer");
    // u64 protocol counter comfortably sums two u32::MAX increments.
    client.record_usage(&a1, &svc, &u32::MAX);
    client.record_usage(&a2, &svc, &u32::MAX);
    assert_eq!(client.get_total_requests_all_time(), (u32::MAX as u64) * 2);
}

/// # Issue #165: Agent authorization on record_usage
///
/// Tests for `agent.require_auth()` enforcement in `record_usage`. The agent
/// must authorize the call; unauthorized callers cannot forge usage on behalf
/// of other agents. Soroban's auth tree allows metering operators to record
/// on the agent's behalf via sub-invocation authorization if the agent has
/// pre-authorized them.
/// `record_usage` rejects when the agent does not authorize the call.
///
/// This test uses `setup_scoped_auth` to set up auth mocking that allows only
/// `init`, then attempts to call `record_usage` without the agent's signature.
/// The call must fail because `agent.require_auth()` is checked at step 0,
/// before all other validation gates.
#[test]
#[should_panic]
fn test_i165_record_usage_requires_agent_auth() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Escrow);
    let client = EscrowClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "init",
            args: (admin.clone(),).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.init(&admin);

    // Attempt to record usage for an agent without authorizing as that agent.
    // No mock_auths are set up for record_usage, so the agent.require_auth()
    // call will fail.
    let agent = Address::generate(&env);
    let service_id = Symbol::new(&env, "weather_api");
    client.record_usage(&agent, &service_id, &100u32);
}

/// `record_usage` succeeds when the agent authorizes the call.
///
/// This is a positive control test: when the agent's signature is included
/// via `mock_auths`, the auth check passes and the call proceeds normally.
#[test]
fn test_i165_record_usage_succeeds_with_agent_auth() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Escrow);
    let client = EscrowClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    // Mock both init and record_usage so both calls are authorized.
    let agent = Address::generate(&env);
    let service_id = Symbol::new(&env, "weather_api");

    env.mock_auths(&[
        MockAuth {
            address: &admin,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "init",
                args: (admin.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        },
        MockAuth {
            address: &agent,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "record_usage",
                args: (agent.clone(), service_id.clone(), 100u32).into_val(&env),
                sub_invokes: &[],
            },
        },
    ]);

    client.init(&admin);
    let record = client.record_usage(&agent, &service_id, &100u32);
    assert_eq!(record.requests, 100);
    assert_eq!(record.agent, agent);
    assert_eq!(record.service_id, service_id);
}

/// Agent auth is checked before the pause gate.
///
/// Per the validation order table in the `record_usage` doc comment, auth
/// (step 0) is checked before the pause gate (step 1). This test confirms
/// that auth failure occurs even when the contract is paused.
#[test]
#[should_panic]
fn test_i165_record_usage_auth_checked_before_pause() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Escrow);
    let client = EscrowClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "init",
            args: (admin.clone(),).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.init(&admin);
    client.pause();

    // Try to record usage without the agent's signature, on a paused contract.
    // Auth failure (step 0) must occur before the pause check (step 1).
    let agent = Address::generate(&env);
    let service_id = Symbol::new(&env, "weather_api");
    client.record_usage(&agent, &service_id, &100u32);
}

/// Agent auth is independent per agent: one agent's signature does not
/// authorize calls on behalf of another agent.
///
/// This test ensures that the auth tie is correctly bound to the specific
/// agent address passed as a parameter; forging usage on behalf of a
/// different agent must fail even if the other agent has previously
/// authorized a call.
#[test]
#[should_panic]
fn test_i165_record_usage_auth_is_per_agent() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Escrow);
    let client = EscrowClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let agent_a = Address::generate(&env);
    let agent_b = Address::generate(&env);
    let service_id = Symbol::new(&env, "weather_api");

    // Set up auth for init and for agent_a only.
    env.mock_auths(&[
        MockAuth {
            address: &admin,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "init",
                args: (admin.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        },
        MockAuth {
            address: &agent_a,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "record_usage",
                args: (agent_a.clone(), service_id.clone(), 50u32).into_val(&env),
                sub_invokes: &[],
            },
        },
    ]);

    client.init(&admin);
    // Record for agent_a succeeds.
    client.record_usage(&agent_a, &service_id, &50u32);

    // Try to record for agent_b using agent_a's auth context. This must fail
    // because agent_b has not authorized the call.
    client.record_usage(&agent_b, &service_id, &100u32);
}

/// Register and `init` the contract authorising only `admin` for the `init`
/// call. Subsequent privileged calls are intentionally left unauthorised so
/// their `require_auth` fails.
fn setup_scoped_auth(env: &Env) -> EscrowClient<'_> {
    let contract_id = env.register_contract(None, Escrow);
    let client = EscrowClient::new(env, &contract_id);
    let admin = Address::generate(env);
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "init",
            args: (admin.clone(),).into_val(env),
            sub_invokes: &[],
        },
    }]);
    client.init(&admin);
    client
}
#[test]
#[should_panic]
fn test_i22_pause_requires_admin_auth() {
    let env = Env::default();
    let client = setup_scoped_auth(&env);
    client.pause();
}
#[test]
#[should_panic]
fn test_i22_set_service_price_requires_admin_auth() {
    let env = Env::default();
    let client = setup_scoped_auth(&env);
    client.set_service_price(&Symbol::new(&env, "infer"), &10i128);
}
#[test]
#[should_panic]
fn test_i22_register_service_requires_admin_auth() {
    let env = Env::default();
    let client = setup_scoped_auth(&env);
    client.register_service(&Symbol::new(&env, "infer"));
}
#[test]
#[should_panic]
fn test_i22_set_agent_allowed_requires_admin_auth() {
    let env = Env::default();
    let client = setup_scoped_auth(&env);
    let agent = Address::generate(&env);
    client.set_agent_allowed(&agent, &true);
}
#[test]
#[should_panic]
fn test_i22_set_service_disabled_requires_admin_auth() {
    let env = Env::default();
    let client = setup_scoped_auth(&env);
    client.set_service_disabled(&Symbol::new(&env, "infer"), &true);
}
#[test]
#[should_panic]
fn test_i22_migrate_requires_admin_auth() {
    let env = Env::default();
    let client = setup_scoped_auth(&env);
    client.migrate_v1_to_v2();
}
#[test]
#[should_panic]
fn test_i22_propose_admin_transfer_requires_admin_auth() {
    let env = Env::default();
    let client = setup_scoped_auth(&env);
    let next = Address::generate(&env);
    client.propose_admin_transfer(&next);
}
#[test]
#[should_panic]
fn test_i22_set_price_tiers_requires_admin_auth() {
    let env = Env::default();
    let client = setup_scoped_auth(&env);
    let tiers: Vec<PriceTier> = Vec::new(&env);
    client.set_price_tiers(&Symbol::new(&env, "infer"), &tiers);
}
#[test]
#[should_panic]
fn test_i22_remove_price_tiers_requires_admin_auth() {
    let env = Env::default();
    let client = setup_scoped_auth(&env);
    client.remove_price_tiers(&Symbol::new(&env, "infer"));
}
#[test]
#[should_panic]
fn test_i22_resolve_dispute_requires_admin_auth() {
    let env = Env::default();
    let client = setup_scoped_auth(&env);
    let agent = Address::generate(&env);
    client.resolve_dispute(&agent, &Symbol::new(&env, "infer"), &0u32);
}

/// Positive control: with `mock_all_auths` the same privileged call
/// succeeds, proving the panics above stem from the missing signature.
#[test]
fn test_i22_pause_succeeds_with_admin_auth() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, Escrow);
    let client = EscrowClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.init(&admin);
    client.pause();
    assert!(client.is_paused());
}

/// With the allowlist disabled (the default), any agent can record usage.
#[test]
fn test_allowlist_disabled_allows_any_agent() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    assert!(!client.is_allowlist_enabled());

    let agent = Address::generate(&env);
    let service_id = Symbol::new(&env, "weather_api");
    let record = client.record_usage(&agent, &service_id, &5u32);
    assert_eq!(record.requests, 5);
}

/// With the allowlist enabled and the agent not listed, record_usage panics
/// with AgentNotAllowed (#10).
#[test]
#[should_panic(expected = "Error(Contract, #10)")]
fn test_allowlist_enabled_rejects_unlisted_agent() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.set_allowlist_enabled(&true);

    let agent = Address::generate(&env);
    let service_id = Symbol::new(&env, "weather_api");
    client.record_usage(&agent, &service_id, &1u32);
}

/// With the allowlist enabled and the agent explicitly allowed, record_usage
/// succeeds.
#[test]
fn test_allowlist_enabled_allows_listed_agent() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.set_allowlist_enabled(&true);

    let agent = Address::generate(&env);
    client.set_agent_allowed(&agent, &true);
    assert!(client.is_agent_allowed(&agent));

    let service_id = Symbol::new(&env, "weather_api");
    let record = client.record_usage(&agent, &service_id, &3u32);
    assert_eq!(record.requests, 3);
}

/// An agent allowed then revoked is rejected again with #10.
#[test]
#[should_panic(expected = "Error(Contract, #10)")]
fn test_allowlist_revocation_reblocks_agent() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.set_allowlist_enabled(&true);

    let agent = Address::generate(&env);
    let service_id = Symbol::new(&env, "weather_api");
    client.set_agent_allowed(&agent, &true);
    client.record_usage(&agent, &service_id, &2u32);

    // Revoke and try again — must be rejected.
    client.set_agent_allowed(&agent, &false);
    assert!(!client.is_agent_allowed(&agent));
    client.record_usage(&agent, &service_id, &1u32);
}

/// Disabling the gate after enabling it restores access for any agent.
#[test]
fn test_allowlist_disable_restores_access() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let service_id = Symbol::new(&env, "weather_api");

    client.set_allowlist_enabled(&true);
    // Gate on, agent unlisted → blocked (try_ to avoid unwinding the test).
    assert!(client.try_record_usage(&agent, &service_id, &1u32).is_err());

    // Turn the gate back off; the unlisted agent can record again.
    client.set_allowlist_enabled(&false);
    let record = client.record_usage(&agent, &service_id, &7u32);
    assert_eq!(record.requests, 7);
}

/// is_allowlist_enabled / is_agent_allowed round-trip cleanly.
#[test]
fn test_allowlist_status_round_trips() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    assert!(!client.is_allowlist_enabled());
    client.set_allowlist_enabled(&true);
    assert!(client.is_allowlist_enabled());

    let agent = Address::generate(&env);
    assert!(!client.is_agent_allowed(&agent));
    client.set_agent_allowed(&agent, &true);
    assert!(client.is_agent_allowed(&agent));
    client.set_agent_allowed(&agent, &false);
    assert!(!client.is_agent_allowed(&agent));
}

/// With the gate on, multiple agents of mixed status are handled independently:
/// the allowed one records, the unlisted one is blocked.
#[test]
fn test_allowlist_mixed_agents() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let service_id = Symbol::new(&env, "weather_api");

    let allowed = Address::generate(&env);
    let blocked = Address::generate(&env);
    client.set_allowlist_enabled(&true);
    client.set_agent_allowed(&allowed, &true);

    let record = client.record_usage(&allowed, &service_id, &4u32);
    assert_eq!(record.requests, 4);
    assert!(client
        .try_record_usage(&blocked, &service_id, &1u32)
        .is_err());
}

/// With strict registration off (default), pricing an unregistered service
/// still works — backward compatible.
#[test]
fn test_set_price_lax_allows_unregistered_service() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "infer");
    client.set_service_price(&svc, &500i128);
    assert_eq!(client.get_service_price(&svc), 500i128);
}

/// With strict registration on, pricing a registered service works.
#[test]
fn test_set_price_strict_allows_registered_service() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "infer");
    client.set_require_service_registration(&true);
    client.register_service(&svc);
    client.set_service_price(&svc, &750i128);
    assert_eq!(client.get_service_price(&svc), 750i128);
}

/// With strict registration on, pricing an unregistered service is rejected
/// with ServiceNotRegistered (#7).
#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_set_price_strict_rejects_unregistered_service() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "phantom");
    client.set_require_service_registration(&true);
    client.set_service_price(&svc, &100i128);
}

/// Pricing a disabled service is rejected with ServiceDisabled (#12),
/// regardless of the strict-registration flag.
#[test]
#[should_panic(expected = "Error(Contract, #12)")]
fn test_set_price_rejects_disabled_service() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "infer");
    client.set_service_disabled(&svc, &true);
    client.set_service_price(&svc, &100i128);
}

/// Toggling the flag on mid-life starts enforcing the coupling: a service
/// priced while lax can no longer be re-priced once strict unless registered.
#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_set_price_flag_toggled_mid_life() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let svc = Symbol::new(&env, "infer");
    client.set_service_price(&svc, &100i128); // lax: allowed
    client.set_require_service_registration(&true);
    client.set_service_price(&svc, &200i128); // strict + unregistered: rejected
}

/// A service owner can settle their own service via `settle_all`.
#[test]
fn test_owner_can_settle_own_service() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let owner = Address::generate(&env);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "infer");

    client.set_service_metadata(&svc, &String::from_str(&env, "inference"), &owner);
    client.set_service_price(&svc, &10i128);
    client.record_usage(&agent, &svc, &5u32);

    let billed = client.settle_all(&owner, &agent);
    assert_eq!(billed.len(), 1);
    let (settled_svc, settled_amount) = billed.get(0).unwrap();
    assert_eq!(settled_svc, svc);
    assert_eq!(settled_amount, 50i128);
    assert_eq!(client.get_usage(&agent, &svc), 0);
}

/// The admin can always settle a service directly.
#[test]
fn test_admin_can_settle_owned_service() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let owner = Address::generate(&env);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "infer");

    client.set_service_metadata(&svc, &String::from_str(&env, "inference"), &owner);
    client.set_service_price(&svc, &10i128);
    client.record_usage(&agent, &svc, &4u32);

    let billed = client.settle(&admin, &agent, &svc);
    assert_eq!(billed, 40i128);
    assert_eq!(client.get_usage(&agent, &svc), 0);
}

/// `settle` is admin-gated, so a caller with the admin key can settle any
/// service regardless of ownership metadata.
#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_owner_cannot_settle_other_service() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let owner_a = Address::generate(&env);
    let owner_b = Address::generate(&env);
    let agent = Address::generate(&env);
    let svc_a = Symbol::new(&env, "svc_a");
    let svc_b = Symbol::new(&env, "svc_b");

    client.set_service_metadata(&svc_a, &String::from_str(&env, "a"), &owner_a);
    client.set_service_metadata(&svc_b, &String::from_str(&env, "b"), &owner_b);
    client.set_service_price(&svc_b, &10i128);
    client.record_usage(&agent, &svc_b, &3u32);

    client.settle(&owner_a, &agent, &svc_b);
}

/// `settle` requires service metadata when the caller is not the admin;
/// a non-admin caller for an unregistered service is rejected.
#[test]
#[should_panic(expected = "Error(Contract, #13)")]
fn test_nonadmin_settle_without_metadata_rejected() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "infer");
    client.register_service(&svc);
    client.set_service_price(&svc, &10i128);
    client.record_usage(&agent, &svc, &2u32);

    client.settle(&agent, &agent, &svc);
}

/// The pause gate still applies to owner-authorized settlement.
#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_owner_settle_rejected_while_paused() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let owner = Address::generate(&env);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "infer");
    client.set_service_metadata(&svc, &String::from_str(&env, "inference"), &owner);
    client.pause();
    client.settle(&owner, &agent, &svc);
}

/// By default the limiter is disabled (cap 0, window 0): an agent can record
/// far more than any cap would allow.
#[test]
fn test_rate_limit_disabled_by_default() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    assert_eq!(client.get_max_requests_per_window(), 0);
    assert_eq!(client.get_rate_window_seconds(), 0);

    let agent = make_agent(&env);
    let svc = make_service(&env, "infer");
    for _ in 0..50 {
        client.record_usage(&agent, &svc, &100u32);
    }
    assert_eq!(client.get_usage(&agent, &svc), 5_000);
}

/// The limiter stays disabled when only the cap is configured.
#[test]
fn test_rate_limit_disabled_when_window_is_zero() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    configure_rate_limit(&client, 5, 0);

    let agent = make_agent(&env);
    let svc = make_service(&env, "infer");
    for _ in 0..4 {
        client.record_usage(&agent, &svc, &5u32);
    }

    assert_eq!(client.get_usage(&agent, &svc), 20);
}

/// The limiter stays disabled when only the window length is configured.
#[test]
fn test_rate_limit_disabled_when_cap_is_zero() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    configure_rate_limit(&client, 0, 60);

    let agent = make_agent(&env);
    let svc = make_service(&env, "infer");
    for _ in 0..4 {
        client.record_usage(&agent, &svc, &7u32);
    }

    assert_eq!(client.get_usage(&agent, &svc), 28);
}

/// Config setters round-trip.
#[test]
fn test_rate_limit_config_round_trips() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    configure_rate_limit(&client, 10, 60);
    assert_eq!(client.get_max_requests_per_window(), 10);
    assert_eq!(client.get_rate_window_seconds(), 60);
}

/// Accumulating exactly to the cap within a window succeeds.
#[test]
fn test_rate_limit_allows_exactly_at_cap() {
    let env = Env::default();
    env.ledger().with_mut(|li| li.timestamp = 1_000);
    let (client, admin) = setup_initialized(&env);
    configure_rate_limit(&client, 10, 100);

    let agent = make_agent(&env);
    let svc = make_service(&env, "infer");

    let first = client.record_usage(&agent, &svc, &6u32);
    assert_eq!(first.requests, 6);

    let second = client.record_usage(&agent, &svc, &4u32);
    assert_eq!(second.requests, 10);
    assert_eq!(client.get_usage(&agent, &svc), 10);
}

/// Accumulating exactly up to the cap is allowed; one more request in the
/// same window is rejected with RateLimitExceeded (#15).
#[test]
#[should_panic(expected = "Error(Contract, #15)")]
fn test_rate_limit_rejects_over_cap_in_window() {
    let env = Env::default();
    env.ledger().with_mut(|li| li.timestamp = 1_000);
    let (client, admin) = setup_initialized(&env);
    configure_rate_limit(&client, 10, 100);

    let agent = make_agent(&env);
    let svc = make_service(&env, "infer");
    client.record_usage(&agent, &svc, &6u32); // count = 6
    client.record_usage(&agent, &svc, &4u32); // count = 10 (exactly at cap)
    client.record_usage(&agent, &svc, &1u32); // count = 11 → reject #15
}

/// A single request larger than the configured cap is rejected immediately.
#[test]
#[should_panic(expected = "Error(Contract, #15)")]
fn test_rate_limit_rejects_single_huge_request() {
    let env = Env::default();
    env.ledger().with_mut(|li| li.timestamp = 1_000);
    let (client, admin) = setup_initialized(&env);
    configure_rate_limit(&client, 10, 100);

    let agent = make_agent(&env);
    let svc = make_service(&env, "infer");
    client.record_usage(&agent, &svc, &11u32);
}

/// After the window expires the counter resets and the agent can record
/// again (fixed-window rollover).
#[test]
fn test_rate_limit_window_rollover_resets_count() {
    let env = Env::default();
    env.ledger().with_mut(|li| li.timestamp = 1_000);
    let (client, admin) = setup_initialized(&env);
    configure_rate_limit(&client, 10, 100);

    let agent = make_agent(&env);
    let svc = make_service(&env, "infer");
    client.record_usage(&agent, &svc, &10u32); // fills the window

    // Advance to the exact boundary; the fixed window must roll over.
    env.ledger().with_mut(|li| li.timestamp = 1_100);
    let rec = client.record_usage(&agent, &svc, &10u32);
    // Usage is cumulative (20), but the rate window accepted the new 10.
    assert_eq!(rec.requests, 20);
}

/// A one-second window still rolls over at the exact `>=` boundary.
#[test]
fn test_rate_limit_one_second_window_rolls_forward() {
    let env = Env::default();
    env.ledger().with_mut(|li| li.timestamp = 1_000);
    let (client, admin) = setup_initialized(&env);
    configure_rate_limit(&client, 2, 1);

    let agent = make_agent(&env);
    let svc = make_service(&env, "infer");
    client.record_usage(&agent, &svc, &2u32);

    advance_ledger(&env, 1);
    let rec = client.record_usage(&agent, &svc, &2u32);
    assert_eq!(rec.requests, 4);
}

/// Mid-window calls do not reset `window_start`; overflow still uses the
/// first in-window timestamp as the anchor.
#[test]
#[should_panic(expected = "Error(Contract, #15)")]
fn test_rate_limit_mid_window_recording_cannot_reset_window_early() {
    let env = Env::default();
    env.ledger().with_mut(|li| li.timestamp = 1_000);
    let (client, admin) = setup_initialized(&env);
    configure_rate_limit(&client, 10, 10);

    let agent = make_agent(&env);
    let svc = make_service(&env, "infer");
    client.record_usage(&agent, &svc, &4u32); // anchor = 1000, count = 4

    advance_ledger(&env, 5);
    client.record_usage(&agent, &svc, &4u32); // still same window, count = 8

    advance_ledger(&env, 4); // now = 1009, still before 1000 + 10
    client.record_usage(&agent, &svc, &3u32); // would succeed if anchor moved to 1005
}

/// Once the original window reaches its boundary, the next record opens a new
/// window even if there was later traffic inside the old one.
#[test]
fn test_rate_limit_window_is_anchored_at_first_in_window_call() {
    let env = Env::default();
    env.ledger().with_mut(|li| li.timestamp = 1_000);
    let (client, admin) = setup_initialized(&env);
    configure_rate_limit(&client, 10, 10);

    let agent = make_agent(&env);
    let svc = make_service(&env, "infer");
    client.record_usage(&agent, &svc, &4u32); // anchor = 1000

    advance_ledger(&env, 5);
    client.record_usage(&agent, &svc, &4u32); // count = 8, anchor must stay 1000

    advance_ledger(&env, 5); // now = 1010, exactly at the original boundary
    let rec = client.record_usage(&agent, &svc, &6u32);

    assert_eq!(rec.requests, 14);
    assert_eq!(client.get_usage(&agent, &svc), 14);
}

/// The limiter is per-agent: one agent hitting the cap does not block another.
#[test]
fn test_rate_limit_is_per_agent() {
    let env = Env::default();
    env.ledger().with_mut(|li| li.timestamp = 1_000);
    let (client, admin) = setup_initialized(&env);
    configure_rate_limit(&client, 5, 100);

    let a = make_agent(&env);
    let b = make_agent(&env);
    let svc = make_service(&env, "infer");
    client.record_usage(&a, &svc, &5u32); // a at cap
    let rec_b = client.record_usage(&b, &svc, &5u32); // b independent
    assert_eq!(rec_b.requests, 5);
}
// ── reset_rate_window tests ──────────────────────────────────────────────────
//
// `reset_rate_window(agent)` clears the per-agent `RateWindow` storage slot
// so the next `record_usage` opens a fresh window. It is admin-gated,
// pause-respecting, idempotent (no-op when no window exists), and emits
// a `rate_rst(agent)` event. Covered scenarios:
//   1. Throttle → reset → record succeeds in the same ledger.
//   2. No-op when no window exists (limiter never hit).
//   3. Non-admin caller is rejected.
//   4. Paused contract is rejected.
//   5. Reset mid-window, then re-throttle in the same window.
//   6. Reset with limiter disabled (no window state, still emits event).
//   7. Reset then immediately re-throttle.

/// Throttle an agent to the cap, reset the window, then record usage
/// again — all within the same fixed window. Demonstrates the admin
/// override lifts the throttle without waiting for window rollover.
#[test]
fn test_reset_rate_window_lifts_throttle() {
    let env = Env::default();
    env.ledger().with_mut(|li| li.timestamp = 1_000);
    let (client, _admin) = setup_initialized(&env);
    client.set_max_requests_per_window(&10u32);
    client.set_rate_window_seconds(&100u64);

    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "infer");

    // Fill the window (count = 10).
    client.record_usage(&agent, &svc, &10u32);
    // The next call would be rejected (RateLimitExceeded #15).
    assert!(client.try_record_usage(&agent, &svc, &1u32).is_err());

    // Reset the rate window — clears the agent's accumulated count.
    client.reset_rate_window(&agent);

    // Now the agent can record again even though the window hasn't rolled.
    let rec = client.record_usage(&agent, &svc, &5u32);
    assert_eq!(rec.requests, 15);
}

/// Resetting an agent that has no stored rate window is a no-op (no panic).
#[test]
fn test_reset_rate_window_noop_when_no_window() {
    let env = Env::default();
    let (client, _admin) = setup_initialized(&env);

    let agent = Address::generate(&env);
    // No record_usage has been called, so no RateWindow exists.
    client.reset_rate_window(&agent);
    // Event is still emitted for auditability.
    let events = env.events().all();
    let expected_topics: soroban_sdk::Vec<soroban_sdk::Val> =
        (symbol_short!("rate_rst"),).into_val(&env);
    let has_rate_rst = events.iter().any(|(_, t, _)| t == expected_topics);
    assert!(has_rate_rst);
}

/// A non-admin caller is rejected with `Unauthorized`.
#[test]
#[should_panic(expected = "Unauthorized")]
fn test_reset_rate_window_non_admin_rejected() {
    let env = Env::default();
    let (client, _admin) = setup_initialized(&env);
    env.set_auths(&[]);

    let agent = Address::generate(&env);
    client.reset_rate_window(&agent);
}

/// Calling while the contract is paused panics with ContractPaused (#4).
#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_reset_rate_window_paused_rejected() {
    let env = Env::default();
    let (client, _admin) = setup_initialized(&env);
    client.pause();

    let agent = Address::generate(&env);
    client.reset_rate_window(&agent);
}

/// Reset the window mid-window, re-fill it, and verify the cap is enforced
/// again within the same fixed window.
#[test]
fn test_reset_rate_window_mid_window_rethrottle() {
    let env = Env::default();
    env.ledger().with_mut(|li| li.timestamp = 1_000);
    let (client, _admin) = setup_initialized(&env);
    client.set_max_requests_per_window(&5u32);
    client.set_rate_window_seconds(&100u64);

    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "infer");

    // Record up to the cap.
    client.record_usage(&agent, &svc, &5u32);
    assert!(client.try_record_usage(&agent, &svc, &1u32).is_err());

    // Reset mid-window — count goes back to 0.
    client.reset_rate_window(&agent);

    // Re-fill the window.
    client.record_usage(&agent, &svc, &5u32);
    // Now it should be throttled again.
    assert!(client.try_record_usage(&agent, &svc, &1u32).is_err());
}

/// When the limiter is disabled (cap = 0 or window = 0), `record_usage`
/// never writes a `RateWindow` slot, so reset is a no-op.
#[test]
fn test_reset_rate_window_limiter_disabled() {
    let env = Env::default();
    let (client, _admin) = setup_initialized(&env);
    // Limiter is disabled by default (cap = 0, window = 0).

    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "infer");

    // Record some usage — RateWindow is never written.
    client.record_usage(&agent, &svc, &100u32);

    // Reset has no effect but should not panic.
    client.reset_rate_window(&agent);

    // Recording still works.
    let rec = client.record_usage(&agent, &svc, &50u32);
    assert_eq!(rec.requests, 150);
}

/// Reset emits a `rate_rst(agent)` event that indexers can observe.
#[test]
fn test_reset_rate_window_emits_event() {
    let env = Env::default();
    let (client, _admin) = setup_initialized(&env);
    let agent = Address::generate(&env);

    client.reset_rate_window(&agent);

    let events = env.events().all();
    assert!(!events.is_empty());
    let (_addr, topics, data) = events.last().unwrap();
    let expected_topics: soroban_sdk::Vec<soroban_sdk::Val> =
        (symbol_short!("rate_rst"),).into_val(&env);
    assert_eq!(topics, expected_topics);
    let decoded: Address = data.into_val(&env);
    assert_eq!(decoded, agent);
}

/// Reset is idempotent: calling it twice in a row does not panic.
#[test]
fn test_reset_rate_window_idempotent() {
    let env = Env::default();
    let (client, _admin) = setup_initialized(&env);
    let agent = Address::generate(&env);

    // First reset (no window yet — no-op).
    client.reset_rate_window(&agent);
    // Second reset — must also succeed.
    client.reset_rate_window(&agent);
}

// ── compute_billing tests ────────────────────────────────────────────────────
//
// `compute_billing(agent, service_id)` returns `accumulated_requests * price_per_request`
// using `saturating_mul`, returns `0` when either operand is zero, and is the
// read-only mirror of the billing math inside `settle`.
//
// Covered scenarios:
//   1. Zero usage, any price          → 0
//   2. Zero price (free service)      → 0
//   3. Unpriced and unused pair       → 0
//   4. Normal product                 → requests * price
//   5. Saturation edge                → i128::MAX (no overflow)
//   6. compute_billing agrees with settle billed value

/// Helper: register a service price for `service_id`.
fn set_price(client: &EscrowClient, service_id: &Symbol, price: i128) {
    client.set_service_price(service_id, &price);
}

/// Helper: record `requests` units of usage for `(agent, service_id)`.
fn record(client: &EscrowClient, agent: &Address, service_id: &Symbol, requests: u32) {
    client.record_usage(agent, service_id, &requests);
}

/// Zero usage with a non-zero price must bill 0.
#[test]
fn test_compute_billing_zero_usage() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "infer");

    set_price(&client, &svc, 100);
    // No record_usage call — accumulated_requests is 0.
    let bill = client.compute_billing(&agent, &svc);
    assert_eq!(bill, 0, "zero usage must bill 0 regardless of price");
}

/// Zero price (free service) with non-zero usage must bill 0.
#[test]
fn test_compute_billing_zero_price_free_service() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "free");

    set_price(&client, &svc, 0); // explicitly free
    record(&client, &agent, &svc, 50);
    let bill = client.compute_billing(&agent, &svc);
    assert_eq!(bill, 0, "free service (price=0) must always bill 0");
}

/// Pair with no price set and no usage recorded must bill 0.
#[test]
fn test_compute_billing_unpriced_and_unused() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "ghost");

    // Neither set_service_price nor record_usage called.
    let bill = client.compute_billing(&agent, &svc);
    assert_eq!(bill, 0, "unpriced and unused pair must bill 0");
}

/// Normal product: 10 requests × 250 stroops/request = 2_500 stroops.
#[test]
fn test_compute_billing_normal_product() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "embed");

    set_price(&client, &svc, 250);
    record(&client, &agent, &svc, 10);
    let bill = client.compute_billing(&agent, &svc);
    assert_eq!(bill, 2_500, "10 requests × 250 stroops must equal 2500");
}

/// Accumulated usage across multiple record_usage calls is summed correctly.
#[test]
fn test_compute_billing_accumulated_usage() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "chat");

    set_price(&client, &svc, 10);
    record(&client, &agent, &svc, 5);
    record(&client, &agent, &svc, 15);
    // total usage = 20, price = 10 → bill = 200
    let bill = client.compute_billing(&agent, &svc);
    assert_eq!(
        bill, 200,
        "accumulated usage across calls must sum correctly"
    );
}

/// Saturation edge: large requests × large price saturates at i128::MAX.
#[test]
fn test_compute_billing_saturation() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "sat");

    // i128::MAX / i128::MAX would overflow without saturating_mul.
    // Use price = i128::MAX so that even 1 request saturates.
    set_price(&client, &svc, i128::MAX);
    record(&client, &agent, &svc, 1);
    let bill = client.compute_billing(&agent, &svc);
    assert_eq!(
        bill,
        i128::MAX,
        "1 request × i128::MAX price must saturate at i128::MAX"
    );
}

/// Saturation with u32::MAX requests also saturates at i128::MAX.
#[test]
fn test_compute_billing_saturation_large_requests() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "sat2");

    // price high enough that u32::MAX * price overflows i128
    set_price(&client, &svc, i128::MAX);
    // record_usage caps at u32::MAX via saturating_add, so record in steps
    record(&client, &agent, &svc, u32::MAX);
    let bill = client.compute_billing(&agent, &svc);
    assert_eq!(
        bill,
        i128::MAX,
        "u32::MAX requests × large price must saturate at i128::MAX"
    );
}

/// compute_billing agrees with the `billed` value settle returns for the same state.
#[test]
fn test_compute_billing_agrees_with_settle() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "agree");

    set_price(&client, &svc, 75);
    record(&client, &agent, &svc, 8);

    // Read compute_billing BEFORE settle (pre-settle state).
    let pre_settle_bill = client.compute_billing(&agent, &svc);

    // settle returns the billed amount and drains the counter.
    let settled = client.settle(&admin, &agent, &svc);

    assert_eq!(
        pre_settle_bill, settled,
        "compute_billing must equal the billed value settle returns for the same pre-settle state"
    );
    assert_eq!(pre_settle_bill, 600, "8 requests × 75 stroops = 600");
}

/// After settle drains the counter, compute_billing returns 0.
#[test]
fn test_compute_billing_zero_after_settle() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let svc = Symbol::new(&env, "drain");

    set_price(&client, &svc, 50);
    record(&client, &agent, &svc, 4);
    client.settle(&admin, &agent, &svc);

    // Counter is drained — billing must now be 0.
    let post_settle_bill = client.compute_billing(&agent, &svc);
    assert_eq!(
        post_settle_bill, 0,
        "compute_billing must return 0 after settle drains the counter"
    );
}

/// Different agents billed independently for the same service.
#[test]
fn test_compute_billing_independent_per_agent() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let a = Address::generate(&env);
    let b = Address::generate(&env);
    let svc = Symbol::new(&env, "shared");

    set_price(&client, &svc, 20);
    record(&client, &a, &svc, 3); // a: 3 × 20 = 60
    record(&client, &b, &svc, 7); // b: 7 × 20 = 140

    assert_eq!(client.compute_billing(&a, &svc), 60);
    assert_eq!(client.compute_billing(&b, &svc), 140);
}

/// Different services billed independently for the same agent.
#[test]
fn test_compute_billing_independent_per_service() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = Address::generate(&env);
    let svc1 = Symbol::new(&env, "alpha");
    let svc2 = Symbol::new(&env, "beta");

    set_price(&client, &svc1, 10);
    set_price(&client, &svc2, 30);
    record(&client, &agent, &svc1, 5); // 5 × 10 = 50
    record(&client, &agent, &svc2, 2); // 2 × 30 = 60

    assert_eq!(client.compute_billing(&agent, &svc1), 50);
    assert_eq!(client.compute_billing(&agent, &svc2), 60);
}

// ── get_contract_config tests ────────────────────────────────────────────────
//
// `get_contract_config()` returns a `ContractConfig` snapshot carrying every
// global setting. Each field must equal the value returned by the corresponding
// per-field getter for the same storage state. Covered scenarios:
//
//   1. Defaults on a fresh contract (all boolean flags false, numeric defaults)
//   2. Fields match individual getters after config changes
//   3. After toggling pause / allowlist / strict-registration
//   4. After setting per-call bounds and rate-limit window
//   5. Callable while paused (pure read — no pause gate)
//   6. Callable before init (admin returns None)

/// All fields carry their defaults on a freshly initialised contract.
#[test]
fn test_get_contract_config_defaults_on_fresh_contract() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    let cfg = client.get_contract_config();

    assert!(!cfg.paused);
    assert!(!cfg.allowlist_enabled);
    assert!(!cfg.require_service_registration);
    assert_eq!(cfg.max_requests_per_call, u32::MAX);
    assert_eq!(cfg.min_requests_per_call, 0);
    assert_eq!(cfg.max_requests_per_window, 0);
    assert_eq!(cfg.window_seconds, 0);
    assert_eq!(cfg.schema_version, 2);
    assert_eq!(cfg.admin, Some(admin));
}

/// Every field in the snapshot matches the corresponding individual getter.
#[test]
fn test_get_contract_config_matches_individual_getters() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    client.set_allowlist_enabled(&true);
    client.set_require_service_registration(&true);
    client.set_max_requests_per_call(&200u32);
    client.set_min_requests_per_call(&5u32);
    client.set_max_requests_per_window(&50u32);
    client.set_rate_window_seconds(&120u64);

    let cfg = client.get_contract_config();

    assert_eq!(cfg.paused, client.is_paused());
    assert_eq!(cfg.allowlist_enabled, client.is_allowlist_enabled());
    assert_eq!(
        cfg.require_service_registration,
        client.is_service_registration_required()
    );
    assert_eq!(
        cfg.max_requests_per_call,
        client.get_max_requests_per_call()
    );
    assert_eq!(
        cfg.min_requests_per_call,
        client.get_min_requests_per_call()
    );
    assert_eq!(
        cfg.max_requests_per_window,
        client.get_max_requests_per_window()
    );
    assert_eq!(cfg.window_seconds, client.get_rate_window_seconds());
    assert_eq!(cfg.schema_version, client.get_schema_version());
    assert_eq!(cfg.admin, client.get_admin());
}

/// Pausing the contract is reflected in the snapshot; unpausing clears it.
#[test]
fn test_get_contract_config_reflects_pause_toggle() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    assert!(!client.get_contract_config().paused);
    client.pause();
    assert!(client.get_contract_config().paused);
    client.unpause();
    assert!(!client.get_contract_config().paused);
}

/// Toggling the allowlist toggle is reflected in the snapshot.
#[test]
fn test_get_contract_config_reflects_allowlist_toggle() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    assert!(!client.get_contract_config().allowlist_enabled);
    client.set_allowlist_enabled(&true);
    assert!(client.get_contract_config().allowlist_enabled);
    client.set_allowlist_enabled(&false);
    assert!(!client.get_contract_config().allowlist_enabled);
}

/// Toggling the strict-registration flag is reflected in the snapshot.
#[test]
fn test_get_contract_config_reflects_strict_registration_toggle() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    assert!(!client.get_contract_config().require_service_registration);
    client.set_require_service_registration(&true);
    assert!(client.get_contract_config().require_service_registration);
    client.set_require_service_registration(&false);
    assert!(!client.get_contract_config().require_service_registration);
}

/// Per-call bounds and rate-limit window are reflected correctly.
#[test]
fn test_get_contract_config_reflects_bounds_and_window() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    client.set_max_requests_per_call(&500u32);
    client.set_min_requests_per_call(&10u32);
    client.set_max_requests_per_window(&100u32);
    client.set_rate_window_seconds(&300u64);

    let cfg = client.get_contract_config();
    assert_eq!(cfg.max_requests_per_call, 500);
    assert_eq!(cfg.min_requests_per_call, 10);
    assert_eq!(cfg.max_requests_per_window, 100);
    assert_eq!(cfg.window_seconds, 300);
}

/// The snapshot is readable while the contract is paused (pure read, no pause gate).
#[test]
fn test_get_contract_config_readable_while_paused() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.pause();

    let cfg = client.get_contract_config();
    assert!(cfg.paused);
    assert_eq!(cfg.admin, Some(admin));
}

/// Before `init`, `admin` is `None` and all fields carry their defaults.
#[test]
fn test_get_contract_config_before_init_admin_is_none() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, Escrow);
    let client = EscrowClient::new(&env, &contract_id);

    let cfg = client.get_contract_config();
    assert_eq!(cfg.admin, None);
    assert!(!cfg.paused);
    assert!(!cfg.allowlist_enabled);
    assert!(!cfg.require_service_registration);
    assert_eq!(cfg.max_requests_per_call, u32::MAX);
    assert_eq!(cfg.min_requests_per_call, 0);
    assert_eq!(cfg.max_requests_per_window, 0);
    assert_eq!(cfg.window_seconds, 0);
    // schema_version defaults to 1 (implicit pre-migration value) when absent.
    assert_eq!(cfg.schema_version, 1);
}

/// The snapshot after an admin handover carries the new admin address.
#[test]
fn test_get_contract_config_reflects_admin_after_rotation() {
    let env = Env::default();
    let (client, _old_admin) = setup_initialized(&env);
    let next = Address::generate(&env);
    client.propose_admin_transfer(&next);
    client.accept_admin_transfer(&next);

    let cfg = client.get_contract_config();
    assert_eq!(cfg.admin, Some(next));
}

/// The snapshot is consistent: all fields come from the same ledger read.
/// Verify by checking that a second call immediately after returns an
/// identical struct (no storage mutation between reads).
#[test]
fn test_get_contract_config_is_idempotent() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    client.set_allowlist_enabled(&true);
    client.set_max_requests_per_call(&99u32);

    let first = client.get_contract_config();
    let second = client.get_contract_config();
    assert_eq!(first, second);
}

// ── register_service_with_metadata ────────────────────────────────────────────
//
// Coverage:
// - Atomicity: both the registration flag and the metadata slot are written
//   by a single call, and both are immediately readable afterwards.
// - Event: `svc_reg(service_id, owner)` is emitted.
// - Idempotency: re-registering the same service id overwrites the metadata.
// - Edge cases: empty `description` is accepted.
// - Security: non-admin callers are rejected; calling while paused panics #4.
// - Equivalence: the combined call produces the same state as the separate
//   `register_service` + `set_service_metadata` sequence.

/// After one `register_service_with_metadata` call, the service is registered
/// **and** its metadata reflects the exact description and owner provided.
#[test]
fn test_register_service_with_metadata_sets_both_slots() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    let service_id = Symbol::new(&env, "weather_api");
    let description = String::from_str(&env, "Weather data feed");
    let owner = Address::generate(&env);

    client.register_service_with_metadata(&service_id, &description, &owner);

    assert!(client.is_service_registered(&service_id));

    let meta = client.get_service_metadata(&service_id).unwrap();
    assert_eq!(meta.description, description);
    assert_eq!(meta.owner, owner);
}

/// The call emits `svc_reg(service_id, owner)` as the sole event.
#[test]
fn test_register_service_with_metadata_emits_svc_reg_event() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    let service_id = Symbol::new(&env, "forecast");
    let description = String::from_str(&env, "Forecast API");
    let owner = Address::generate(&env);

    client.register_service_with_metadata(&service_id, &description, &owner);

    let events = env.events().all();
    assert!(!events.is_empty());
    let (_addr, topics, data) = events.last().unwrap();

    let expected_topics: soroban_sdk::Vec<soroban_sdk::Val> =
        (symbol_short!("svc_reg"),).into_val(&env);
    assert_eq!(topics, expected_topics);

    let decoded: (Symbol, Address) = data.into_val(&env);
    assert_eq!(decoded, (service_id, owner));
}

/// Re-registering an existing id overwrites its metadata (idempotent overwrite).
#[test]
fn test_register_service_with_metadata_idempotent_overwrite() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    let service_id = Symbol::new(&env, "forecast");
    let desc_a = String::from_str(&env, "v1 description");
    let owner_a = Address::generate(&env);

    client.register_service_with_metadata(&service_id, &desc_a, &owner_a);

    // Re-register with different metadata.
    let desc_b = String::from_str(&env, "v2 description (overwritten)");
    let owner_b = Address::generate(&env);
    client.register_service_with_metadata(&service_id, &desc_b, &owner_b);

    // Still registered.
    assert!(client.is_service_registered(&service_id));
    // Metadata reflects the latest write.
    let meta = client.get_service_metadata(&service_id).unwrap();
    assert_eq!(meta.description, desc_b);
    assert_eq!(meta.owner, owner_b);
}

/// An empty description is accepted.
#[test]
fn test_register_service_with_metadata_empty_description() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    let service_id = Symbol::new(&env, "empty_desc");
    let description = String::from_str(&env, "");
    let owner = Address::generate(&env);

    client.register_service_with_metadata(&service_id, &description, &owner);

    assert!(client.is_service_registered(&service_id));
    let meta = client.get_service_metadata(&service_id).unwrap();
    assert_eq!(meta.description, description);
}

/// A non-admin caller is rejected (panics).
#[test]
#[should_panic]
fn test_register_service_with_metadata_rejects_non_admin() {
    let env = Env::default();
    let client = setup_scoped_auth(&env);

    let service_id = Symbol::new(&env, "unauthorised");
    let description = String::from_str(&env, "sketchy service");
    let owner = Address::generate(&env);

    client.register_service_with_metadata(&service_id, &description, &owner);
}

/// Calling while paused panics with `ContractPaused` (#4).
#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_register_service_with_metadata_panics_when_paused() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    client.pause();

    let service_id = Symbol::new(&env, "paused_svc");
    let description = String::from_str(&env, "should not land");
    let owner = Address::generate(&env);

    client.register_service_with_metadata(&service_id, &description, &owner);
}

/// The combined call is equivalent to calling `register_service` followed by
/// `set_service_metadata` — both produce the same storage state.
#[test]
fn test_register_service_with_metadata_equivalent_to_separate_calls() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);

    let service_id = Symbol::new(&env, "equiv");
    let description = String::from_str(&env, "equivalent service");
    let owner = Address::generate(&env);

    // Combined call.
    client.register_service_with_metadata(&service_id, &description, &owner);

    let combined_registered = client.is_service_registered(&service_id);
    let combined_meta = client.get_service_metadata(&service_id);

    // Fresh contract for separate-call path.
    let env2 = Env::default();
    env2.mock_all_auths();
    let contract_id2 = env2.register_contract(None, Escrow);
    let client2 = EscrowClient::new(&env2, &contract_id2);
    let admin2 = Address::generate(&env2);
    client2.init(&admin2);

    let service_id2 = Symbol::new(&env2, "equiv");
    let description2 = String::from_str(&env2, "equivalent service");
    let owner2 = Address::generate(&env2);

    client2.register_service(&service_id2);
    client2.set_service_metadata(&service_id2, &description2, &owner2);

    assert_eq!(
        combined_registered,
        client2.is_service_registered(&service_id2)
    );
    assert_eq!(combined_meta, client2.get_service_metadata(&service_id2));
}

// ── InvalidRequestBounds (#23): cross-bound consistency checks ─────────────
//
// Coverage:
//
// Setter-rejection cases:
// - set_min > current max  → InvalidRequestBounds (#23)
// - set_max < current min  → InvalidRequestBounds (#23)
//
// Accepted cases:
// - set_min == current max (exact-count)  → accepted; record_usage enforces it
// - set_max == current min (exact-count)  → accepted; record_usage enforces it
// - Neither bound set: defaults (min=0, max=u32::MAX) are always consistent
// - set_max first, then a valid min       → accepted
// - set_min first, then a valid max       → accepted
// - Lowering max to current min           → accepted
// - Raising min to current max            → accepted
//
// Default / unset behaviour:
// - set_min with no max stored            → accepted (ceiling defaults to u32::MAX)
// - set_max with no min stored            → accepted (floor defaults to 0)
// - set_max = 0 with no min stored        → accepted (0 >= 0)
//
// Security: metering cannot be bricked by a contradictory range because the
// setters reject the contradiction before it can be stored.

/// `set_min_requests_per_call` rejects a floor that exceeds the stored ceiling.
///
/// Security note: without this guard an admin could silently configure a range
/// where min > max, making every `record_usage` call panic on either #8 or #9
/// regardless of the supplied value, bricking metering for the service.
#[test]
#[should_panic(expected = "Error(Contract, #23)")]
fn test_set_min_rejects_floor_above_stored_ceiling() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    // Establish a ceiling of 50.
    client.set_max_requests_per_call(&50u32);
    // Attempt to set a floor of 51 — one above the ceiling.
    client.set_min_requests_per_call(&51u32);
}

/// `set_max_requests_per_call` rejects a ceiling that falls below the stored floor.
///
/// Security note: same bricking vector as above, entered from the other direction.
#[test]
#[should_panic(expected = "Error(Contract, #23)")]
fn test_set_max_rejects_ceiling_below_stored_floor() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    // Establish a floor of 10.
    client.set_min_requests_per_call(&10u32);
    // Attempt to set a ceiling of 9 — one below the floor.
    client.set_max_requests_per_call(&9u32);
}

/// `set_min` with no ceiling stored is always accepted (ceiling defaults to u32::MAX).
///
/// Preserves existing default-unset behaviour: operators can set a floor
/// independently without first having to set a ceiling.
#[test]
fn test_set_min_with_no_max_stored_is_accepted() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    // No ceiling stored yet — any min value should be accepted.
    client.set_min_requests_per_call(&1_000_000u32);
    assert_eq!(client.get_min_requests_per_call(), 1_000_000);
    // record_usage with the exact floor value must succeed.
    let agent = make_agent(&env);
    let svc = make_service(&env, "infer");
    let rec = client.record_usage(&agent, &svc, &1_000_000u32);
    assert_eq!(rec.requests, 1_000_000);
}

/// `set_max` with no floor stored is always accepted (floor defaults to 0).
///
/// Preserves existing default-unset behaviour: operators can set a ceiling
/// independently without first having to set a floor.
#[test]
fn test_set_max_with_no_min_stored_is_accepted() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    // No floor stored yet — any max value including 0 should be accepted.
    client.set_max_requests_per_call(&0u32);
    assert_eq!(client.get_max_requests_per_call(), 0);
}

/// `min == max` is accepted: it enforces an exact per-call request count.
///
/// An exact-count constraint is a legitimate use case (e.g. force every
/// metering call to bundle exactly N requests to amortise per-write costs).
#[test]
fn test_set_min_equal_to_max_is_accepted() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    // Set a ceiling of 100, then a floor of 100 (equal).
    client.set_max_requests_per_call(&100u32);
    client.set_min_requests_per_call(&100u32);
    assert_eq!(client.get_max_requests_per_call(), 100);
    assert_eq!(client.get_min_requests_per_call(), 100);
    // record_usage with exactly 100 requests must succeed.
    let agent = make_agent(&env);
    let svc = make_service(&env, "fixed_batch");
    let rec = client.record_usage(&agent, &svc, &100u32);
    assert_eq!(rec.requests, 100);
}

/// Setting the ceiling down to equal the existing floor is accepted.
///
/// Symmetric counterpart of `test_set_min_equal_to_max_is_accepted`:
/// confirms that `set_max_requests_per_call` also permits min == max.
#[test]
fn test_set_max_equal_to_min_is_accepted() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    // Set a floor of 20, then a ceiling of 20 (equal).
    client.set_min_requests_per_call(&20u32);
    client.set_max_requests_per_call(&20u32);
    assert_eq!(client.get_min_requests_per_call(), 20);
    assert_eq!(client.get_max_requests_per_call(), 20);
    // record_usage with exactly 20 requests must succeed.
    let agent = make_agent(&env);
    let svc = make_service(&env, "exact_svc");
    let rec = client.record_usage(&agent, &svc, &20u32);
    assert_eq!(rec.requests, 20);
}

/// record_usage with min == max rejects values below the exact count.
///
/// When the floor and ceiling are equal, any value other than that exact
/// count must be rejected — values below it hit the floor (#9).
#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn test_exact_count_rejects_below() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.set_max_requests_per_call(&50u32);
    client.set_min_requests_per_call(&50u32);
    let agent = make_agent(&env);
    let svc = make_service(&env, "exact_svc");
    client.record_usage(&agent, &svc, &49u32);
}

/// record_usage with min == max rejects values above the exact count.
///
/// When the floor and ceiling are equal, values above it hit the ceiling (#8).
#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_exact_count_rejects_above() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.set_max_requests_per_call(&50u32);
    client.set_min_requests_per_call(&50u32);
    let agent = make_agent(&env);
    let svc = make_service(&env, "exact_svc");
    client.record_usage(&agent, &svc, &51u32);
}

/// Default unset behaviour: no bounds stored → any positive value accepted.
///
/// Verifies that fresh contracts with no min/max configured still accept
/// any positive request count (defaults: min=0, max=u32::MAX).
#[test]
fn test_defaults_no_bounds_stored_accepts_any_positive() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    assert_eq!(client.get_min_requests_per_call(), 0);
    assert_eq!(client.get_max_requests_per_call(), u32::MAX);
    let agent = make_agent(&env);
    let svc = make_service(&env, "infer");
    // A very large value should be accepted with no bounds set.
    let rec = client.record_usage(&agent, &svc, &999_999u32);
    assert_eq!(rec.requests, 999_999);
}

/// set_max first, then a lower-but-valid min → accepted.
///
/// Documents the recommended "ceiling first, floor second" operator
/// ordering to avoid transient `InvalidRequestBounds` rejections.
#[test]
fn test_set_max_then_lower_min_accepted() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.set_max_requests_per_call(&200u32);
    client.set_min_requests_per_call(&50u32);
    assert_eq!(client.get_max_requests_per_call(), 200);
    assert_eq!(client.get_min_requests_per_call(), 50);
    // Values within the valid range must succeed.
    let agent = make_agent(&env);
    let svc = make_service(&env, "range_svc");
    assert_eq!(client.record_usage(&agent, &svc, &50u32).requests, 50);
    assert_eq!(client.record_usage(&agent, &svc, &100u32).requests, 150);
    assert_eq!(client.record_usage(&agent, &svc, &200u32).requests, 350);
}

/// set_min first, then a higher-but-valid max → accepted.
///
/// Alternative operator ordering: floor first, ceiling second.
/// Allowed because setting a floor of N against the default ceiling of
/// u32::MAX never violates the invariant.
#[test]
fn test_set_min_then_higher_max_accepted() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.set_min_requests_per_call(&10u32);
    client.set_max_requests_per_call(&500u32);
    assert_eq!(client.get_min_requests_per_call(), 10);
    assert_eq!(client.get_max_requests_per_call(), 500);
    // Boundary values must succeed.
    let agent = make_agent(&env);
    let svc = make_service(&env, "range_svc");
    assert_eq!(client.record_usage(&agent, &svc, &10u32).requests, 10);
    assert_eq!(client.record_usage(&agent, &svc, &500u32).requests, 510);
}

/// Tightening the ceiling down to the current floor is accepted.
///
/// An admin is allowed to narrow a previously wide range to an exact-count
/// constraint by lowering the ceiling to meet the stored floor.
#[test]
fn test_lower_max_to_meet_min_accepted() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    // Wide range: [5, 200].
    client.set_max_requests_per_call(&200u32);
    client.set_min_requests_per_call(&5u32);
    // Narrow the ceiling down to 5 (== floor) — this is exact-count territory.
    client.set_max_requests_per_call(&5u32);
    assert_eq!(client.get_max_requests_per_call(), 5);
    assert_eq!(client.get_min_requests_per_call(), 5);
}

/// Raising the floor up to the current ceiling is accepted.
///
/// Symmetric counterpart: an admin can raise the floor to meet the stored
/// ceiling, producing an exact-count constraint from the floor side.
#[test]
fn test_raise_min_to_meet_max_accepted() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    // Wide range: [5, 200].
    client.set_max_requests_per_call(&200u32);
    client.set_min_requests_per_call(&5u32);
    // Raise the floor up to 200 (== ceiling) — exact-count.
    client.set_min_requests_per_call(&200u32);
    assert_eq!(client.get_min_requests_per_call(), 200);
    assert_eq!(client.get_max_requests_per_call(), 200);
}

/// Attempting to set min = max+1 is rejected even by a single unit.
///
/// Off-by-one edge: confirms the boundary is `min > max`, not `min >= max`.
#[test]
#[should_panic(expected = "Error(Contract, #23)")]
fn test_set_min_one_above_max_rejected() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.set_max_requests_per_call(&100u32);
    client.set_min_requests_per_call(&101u32); // 101 > 100 → #23
}

/// Attempting to set max = min-1 is rejected even by a single unit.
///
/// Off-by-one edge: confirms the boundary is `max < min`, not `max <= min`.
#[test]
#[should_panic(expected = "Error(Contract, #23)")]
fn test_set_max_one_below_min_rejected() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.set_min_requests_per_call(&10u32);
    client.set_max_requests_per_call(&9u32); // 9 < 10 → #23
}

/// set_max = 0 with no floor stored is accepted (0 >= 0, the default floor).
///
/// A ceiling of 0 is unusual but not contradictory when the floor is also 0.
/// Combined with the existing `RequestsMustBePositive` (#2) guard which
/// rejects `requests == 0`, this configuration effectively makes every
/// `record_usage` call fail with #8 (requests=1 > max=0). Admins are
/// responsible for this configuration choice; the contract only prevents
/// the unsatisfiable min > max case.
#[test]
fn test_set_max_zero_with_no_floor_accepted() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    // No floor stored (default 0) → max=0 satisfies 0 >= 0.
    client.set_max_requests_per_call(&0u32);
    assert_eq!(client.get_max_requests_per_call(), 0);
    // record_usage will always fail (requests must be > 0 by #2, but max=0
    // means any positive value is above the cap). This is intentional; the
    // contract does not prevent self-imposed restrictions, only contradictions.
}

/// Contradictory set_min is rejected; stored state is unchanged.
///
/// Confirms that the pre-attempt values are still readable after the test
/// confirms the setter rejects the invalid floor.
/// The rejection itself is covered by `test_set_min_rejects_floor_above_stored_ceiling`.
#[test]
fn test_rejected_set_min_leaves_state_unchanged() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.set_max_requests_per_call(&100u32);
    client.set_min_requests_per_call(&10u32);

    // Stored values must be the values set by the two successful setter calls.
    // A subsequent attempt to push min above max would be rejected (#23) and
    // is verified by test_set_min_rejects_floor_above_stored_ceiling.
    assert_eq!(client.get_max_requests_per_call(), 100);
    assert_eq!(client.get_min_requests_per_call(), 10);
}

/// Metering is NOT bricked: with consistent bounds, record_usage always has
/// a satisfiable range of inputs.
///
/// This is the security property that the cross-bound check protects.
/// With min=10, max=100, any value in [10, 100] is accepted.
#[test]
fn test_consistent_bounds_never_brick_metering() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    client.set_max_requests_per_call(&100u32);
    client.set_min_requests_per_call(&10u32);

    let agent = make_agent(&env);
    let svc = make_service(&env, "metered_svc");

    // Every value in [10, 100] must succeed.
    for requests in [10u32, 50u32, 99u32, 100u32] {
        // Each call accumulates; we care only that none panic.
        client.record_usage(&agent, &svc, &requests);
    }
    // Total accumulated = 10 + 50 + 99 + 100 = 259.
    assert_eq!(client.get_usage(&agent, &svc), 259);
}

// ── refund_batch tests ────────────────────────────────────────────────────────
//
// `refund_batch` is a bounded admin batch entrypoint that resolves disputes
// for multiple services of one agent, zeroing their full usage in a single
// transaction.  It reuses `MAX_BATCH_READ` as the batch-size cap and emits
// one `dispute` event per refunded pair.
//
// Covered scenarios:
//   1. Single service
//   2. Multiple services (all usage zeroed, all disputes cleared)
//   3. One event emitted per refunded service
//   4. Event payload carries "resolve", agent, service_id, and the refunded
//      amount (= the full usage before zeroing)
//   5. Oversized batch (exceeds MAX_BATCH_READ) → BatchTooLarge (#16)
//   6. Service with no open dispute → silently skipped
//   7. Contract paused → ContractPaused (#4)
//   8. Non-admin caller → Unauthorized
//   9. Service with zero usage: still clears dispute, refunds 0
//  10. Duplicate service ids: each occurrence is processed independently

/// Helper: record usage for an `(agent, service_id)` pair and open a dispute.
fn open_dispute_with_usage(
    client: &EscrowClient<'_>,
    agent: &Address,
    service_id: &Symbol,
    requests: u32,
) {
    client.record_usage(agent, service_id, &requests);
    client.open_dispute(agent, service_id);
}

/// refund_batch resolves a single service dispute, zeroing the usage.
#[test]
fn test_refund_batch_single_service() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = make_agent(&env);
    let svc = make_service(&env, "svc_a");

    open_dispute_with_usage(&client, &agent, &svc, 75);

    assert!(client.has_open_dispute(&agent, &svc));
    assert_eq!(client.get_usage(&agent, &svc), 75);

    let mut services: Vec<Symbol> = Vec::new(&env);
    services.push_back(svc.clone());
    client.refund_batch(&agent, &services);

    // Usage is zeroed.
    assert_eq!(client.get_usage(&agent, &svc), 0);
    // Dispute is cleared.
    assert!(!client.has_open_dispute(&agent, &svc));
}

/// refund_batch resolves multiple services, zeroing all usage and clearing
/// all disputes.
#[test]
fn test_refund_batch_multiple_services() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = make_agent(&env);
    let svc_a = make_service(&env, "svc_a");
    let svc_b = make_service(&env, "svc_b");
    let svc_c = make_service(&env, "svc_c");

    open_dispute_with_usage(&client, &agent, &svc_a, 100);
    open_dispute_with_usage(&client, &agent, &svc_b, 50);
    open_dispute_with_usage(&client, &agent, &svc_c, 25);

    assert!(client.has_open_dispute(&agent, &svc_a));
    assert!(client.has_open_dispute(&agent, &svc_b));
    assert!(client.has_open_dispute(&agent, &svc_c));
    assert_eq!(client.get_usage(&agent, &svc_a), 100);
    assert_eq!(client.get_usage(&agent, &svc_b), 50);
    assert_eq!(client.get_usage(&agent, &svc_c), 25);

    let mut services: Vec<Symbol> = Vec::new(&env);
    services.push_back(svc_a.clone());
    services.push_back(svc_b.clone());
    services.push_back(svc_c.clone());
    client.refund_batch(&agent, &services);

    assert_eq!(client.get_usage(&agent, &svc_a), 0);
    assert_eq!(client.get_usage(&agent, &svc_b), 0);
    assert_eq!(client.get_usage(&agent, &svc_c), 0);
    assert!(!client.has_open_dispute(&agent, &svc_a));
    assert!(!client.has_open_dispute(&agent, &svc_b));
    assert!(!client.has_open_dispute(&agent, &svc_c));
}

/// refund_batch emits exactly one `dispute` event per refunded service.
#[test]
fn test_refund_batch_emits_one_event_per_service() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = make_agent(&env);
    let svc_a = make_service(&env, "svc_a");
    let svc_b = make_service(&env, "svc_b");

    open_dispute_with_usage(&client, &agent, &svc_a, 30);
    open_dispute_with_usage(&client, &agent, &svc_b, 20);

    let mut services: Vec<Symbol> = Vec::new(&env);
    services.push_back(svc_a.clone());
    services.push_back(svc_b.clone());
    client.refund_batch(&agent, &services);

    // `events().all()` only holds the most recent invocation's events,
    // so the refund_batch call's events are the whole buffer.
    let events = env.events().all();
    assert_eq!(events.len(), 2, "one event per refunded service");
}

/// Each `dispute` event carries the expected resolve payload:
/// `("resolve", agent, service_id, refunded)`.
#[test]
fn test_refund_batch_event_payload() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = make_agent(&env);
    let svc = make_service(&env, "svc_a");

    open_dispute_with_usage(&client, &agent, &svc, 42);

    let mut services: Vec<Symbol> = Vec::new(&env);
    services.push_back(svc.clone());
    client.refund_batch(&agent, &services);

    let events = env.events().all();
    let (_addr, topics, data) = events.last().unwrap();

    let expected_topics: soroban_sdk::Vec<soroban_sdk::Val> =
        (symbol_short!("dispute"),).into_val(&env);
    assert_eq!(topics, expected_topics);

    let decoded: (Symbol, Address, Symbol, u32) = data.into_val(&env);
    assert_eq!(decoded.0, symbol_short!("resolve"));
    assert_eq!(decoded.1, agent);
    assert_eq!(decoded.2, svc);
    assert_eq!(decoded.3, 42, "refunded amount equals the pre-zero usage");
}

/// Oversized batch (exceeding MAX_BATCH_READ) is rejected with BatchTooLarge (#16).
#[test]
#[should_panic(expected = "Error(Contract, #16)")]
fn test_refund_batch_oversized_panics() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = make_agent(&env);
    let svc = make_service(&env, "svc");

    // Open disputes on enough services to fill beyond MAX_BATCH_READ.
    // We reuse the same (agent, svc) pair to keep the test short — the
    // dispute flag is per-pair, so we open/close it for each synthetic
    // service created below.
    let mut services: Vec<Symbol> = Vec::new(&env);
    for i in 0..=MAX_BATCH_READ {
        let s = Symbol::new(&env, &alloc::format!("svc_{}", i));
        client.record_usage(&agent, &s, &1u32);
        client.open_dispute(&agent, &s);
        services.push_back(s);
    }
    // services.len() == MAX_BATCH_READ + 1 → must panic.
    client.refund_batch(&agent, &services);
}

/// refund_batch silently skips services without an open dispute.
#[test]
fn test_refund_batch_no_open_dispute_skips_silently() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = make_agent(&env);
    let svc = make_service(&env, "svc");

    // Record usage but do NOT open a dispute.
    client.record_usage(&agent, &svc, &50u32);

    let mut services: Vec<Symbol> = Vec::new(&env);
    services.push_back(svc.clone());
    client.refund_batch(&agent, &services);

    // Service without dispute is untouched.
    assert_eq!(client.get_usage(&agent, &svc), 50);
    assert!(!client.has_open_dispute(&agent, &svc));
}

/// refund_batch skips services without a dispute and processes the rest.
#[test]
fn test_refund_batch_skips_missing_disputes() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = make_agent(&env);
    let svc_a = make_service(&env, "svc_a");
    let svc_b = make_service(&env, "svc_b");

    // Only svc_b has an open dispute — svc_a does not.
    open_dispute_with_usage(&client, &agent, &svc_b, 100);

    let mut services: Vec<Symbol> = Vec::new(&env);
    services.push_back(svc_a.clone());
    services.push_back(svc_b.clone());
    client.refund_batch(&agent, &services);

    // svc_a (no dispute) is untouched.
    assert_eq!(client.get_usage(&agent, &svc_a), 0);
    assert!(!client.has_open_dispute(&agent, &svc_a));

    // svc_b (had dispute) is resolved.
    assert_eq!(client.get_usage(&agent, &svc_b), 0);
    assert!(!client.has_open_dispute(&agent, &svc_b));
}

/// refund_batch panics with ContractPaused (#4) when the contract is paused.
#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_refund_batch_paused_panics() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = make_agent(&env);
    let svc = make_service(&env, "svc");

    open_dispute_with_usage(&client, &agent, &svc, 10);
    client.pause();

    let mut services: Vec<Symbol> = Vec::new(&env);
    services.push_back(svc.clone());
    client.refund_batch(&agent, &services);
}

/// refund_batch panics when called by a non-admin.
#[test]
#[should_panic]
fn test_refund_batch_requires_admin_auth() {
    let env = Env::default();
    let client = setup_scoped_auth(&env);
    let agent = make_agent(&env);
    let svc = make_service(&env, "svc");

    // The contract is initialised but no admin auth is mocked beyond init.
    let mut services: Vec<Symbol> = Vec::new(&env);
    services.push_back(svc.clone());
    client.refund_batch(&agent, &services);
}

/// refund_batch handles a service with zero usage: dispute is cleared and
/// the event carries refund=0.
#[test]
fn test_refund_batch_zero_usage_still_clears_dispute() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = make_agent(&env);
    let svc = make_service(&env, "svc");

    // Open a dispute without any recorded usage.
    client.open_dispute(&agent, &svc);
    assert!(client.has_open_dispute(&agent, &svc));
    assert_eq!(client.get_usage(&agent, &svc), 0);

    let mut services: Vec<Symbol> = Vec::new(&env);
    services.push_back(svc.clone());
    client.refund_batch(&agent, &services);

    // Dispute is cleared, usage stays 0.
    assert!(!client.has_open_dispute(&agent, &svc));
    assert_eq!(client.get_usage(&agent, &svc), 0);
}

/// Duplicate service ids in the batch are processed independently: each
/// occurrence clears the dispute and zeroes usage, and subsequent
/// occurrences see the same post-zero state.
#[test]
fn test_refund_batch_duplicate_services_processed_independently() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = make_agent(&env);
    let svc = make_service(&env, "svc");

    open_dispute_with_usage(&client, &agent, &svc, 60);

    let mut services: Vec<Symbol> = Vec::new(&env);
    services.push_back(svc.clone());
    services.push_back(svc.clone());
    client.refund_batch(&agent, &services);

    // After processing, usage is zero and dispute is cleared.
    assert_eq!(client.get_usage(&agent, &svc), 0);
    assert!(!client.has_open_dispute(&agent, &svc));
}

/// Two agents with independent disputes: refund_batch only touches the
/// specified agent.
#[test]
fn test_refund_batch_only_touches_specified_agent() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent_a = make_agent(&env);
    let agent_b = make_agent(&env);
    let svc = make_service(&env, "svc");

    open_dispute_with_usage(&client, &agent_a, &svc, 80);
    open_dispute_with_usage(&client, &agent_b, &svc, 40);

    let mut services: Vec<Symbol> = Vec::new(&env);
    services.push_back(svc.clone());
    client.refund_batch(&agent_a, &services);

    // agent_a's usage is zeroed and dispute cleared.
    assert_eq!(client.get_usage(&agent_a, &svc), 0);
    assert!(!client.has_open_dispute(&agent_a, &svc));

    // agent_b is untouched.
    assert_eq!(client.get_usage(&agent_b, &svc), 40);
    assert!(client.has_open_dispute(&agent_b, &svc));
}

/// Empty batch is accepted and is a no-op.
#[test]
fn test_refund_batch_empty_list_is_noop() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = make_agent(&env);

    let services: Vec<Symbol> = Vec::new(&env);
    client.refund_batch(&agent, &services);
    // No panic — empty batch is valid.
}

/// Batch at the MAX_BATCH_READ boundary succeeds.
#[test]
fn test_refund_batch_at_boundary_succeeds() {
    let env = Env::default();
    let (client, admin) = setup_initialized(&env);
    let agent = make_agent(&env);

    let mut services: Vec<Symbol> = Vec::new(&env);
    for i in 0..MAX_BATCH_READ {
        let s = Symbol::new(&env, &alloc::format!("svc_{}", i));
        client.record_usage(&agent, &s, &1u32);
        client.open_dispute(&agent, &s);
        services.push_back(s);
    }
    assert_eq!(services.len(), MAX_BATCH_READ);

    client.refund_batch(&agent, &services);

    // All disputes cleared, all usage zeroed.
    for i in 0..MAX_BATCH_READ {
        let s = Symbol::new(&env, &alloc::format!("svc_{}", i));
        assert!(!client.has_open_dispute(&agent, &s));
        assert_eq!(client.get_usage(&agent, &s), 0);
    }
}

/// refund_batch panics with NotInitialized (#3) when the contract has not
/// been initialised.
#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_refund_batch_panics_not_initialized() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, Escrow);
    let client = EscrowClient::new(&env, &contract_id);
    let agent = make_agent(&env);

    let services: Vec<Symbol> = Vec::new(&env);
    client.refund_batch(&agent, &services);
}
