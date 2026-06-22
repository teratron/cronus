use cronus::inbox::{
    actor_exists, drain, gc, migrate, register_actor, send, undrained_count, BusEvent,
    CaptureBusSender, DrainedMessage, InboxError, InboxMessage, NoOpBusSender,
    GC_TTL_MS, MAX_DRAIN_PER_TURN,
};
use rusqlite::Connection;

fn open() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    migrate(&conn).unwrap();
    conn
}

fn now_ms() -> u64 {
    // Stable timestamp for tests — use 0-based epoch offsets.
    1_000_000_000_000
}

// ── Actor registry ────────────────────────────────────────────────────────────

#[test]
fn register_actor_then_exists() {
    let conn = open();
    register_actor(&conn, "agent-a").unwrap();
    assert!(actor_exists(&conn, "agent-a").unwrap());
}

#[test]
fn unregistered_actor_does_not_exist() {
    let conn = open();
    assert!(!actor_exists(&conn, "ghost").unwrap());
}

#[test]
fn register_actor_is_idempotent() {
    let conn = open();
    register_actor(&conn, "a").unwrap();
    register_actor(&conn, "a").unwrap(); // no error
    assert!(actor_exists(&conn, "a").unwrap());
}

// ── Send ──────────────────────────────────────────────────────────────────────

#[test]
fn send_inserts_row_when_sender_registered() {
    let conn = open();
    register_actor(&conn, "sender").unwrap();
    let bus = NoOpBusSender;
    send(
        &conn,
        &InboxMessage {
            sender_id: "sender".into(),
            recipient_id: "recipient".into(),
            body: "hello".into(),
        },
        now_ms(),
        &bus,
    )
    .unwrap();

    let count = undrained_count(&conn, "recipient").unwrap();
    assert_eq!(count, 1);
}

#[test]
fn send_returns_esrch_for_unregistered_sender() {
    let conn = open();
    let bus = NoOpBusSender;
    let result = send(
        &conn,
        &InboxMessage {
            sender_id: "ghost".into(),
            recipient_id: "r".into(),
            body: "msg".into(),
        },
        now_ms(),
        &bus,
    );
    assert!(
        matches!(result, Err(InboxError::SenderNotFound(ref s)) if s == "ghost"),
        "expected SenderNotFound"
    );
}

#[test]
fn send_fires_inbox_arrived_bus_event() {
    let conn = open();
    register_actor(&conn, "s").unwrap();
    let bus = CaptureBusSender::new();
    send(
        &conn,
        &InboxMessage {
            sender_id: "s".into(),
            recipient_id: "r".into(),
            body: "msg".into(),
        },
        now_ms(),
        &bus,
    )
    .unwrap();

    let events = bus.captured();
    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0],
        BusEvent::InboxArrived {
            recipient_id: "r".into(),
            count: 1
        }
    );
}

#[test]
fn send_bus_event_count_increases_with_multiple_sends() {
    let conn = open();
    register_actor(&conn, "s").unwrap();
    let bus = CaptureBusSender::new();
    for _ in 0..3 {
        send(
            &conn,
            &InboxMessage {
                sender_id: "s".into(),
                recipient_id: "r".into(),
                body: "x".into(),
            },
            now_ms(),
            &bus,
        )
        .unwrap();
    }
    let events = bus.captured();
    // Count field in the last event should be 3
    assert!(matches!(&events[2], BusEvent::InboxArrived { count: 3, .. }));
}

// ── Drain ─────────────────────────────────────────────────────────────────────

#[test]
fn drain_returns_messages_in_order() {
    let conn = open();
    register_actor(&conn, "s").unwrap();
    let bus = NoOpBusSender;
    let now = now_ms();
    for body in &["first", "second", "third"] {
        send(
            &conn,
            &InboxMessage {
                sender_id: "s".into(),
                recipient_id: "r".into(),
                body: body.to_string(),
            },
            now,
            &bus,
        )
        .unwrap();
    }

    let msgs: Vec<DrainedMessage> = drain(&conn, "r", now + 1).unwrap();
    assert_eq!(msgs.len(), 3);
    assert_eq!(msgs[0].body, "first");
    assert_eq!(msgs[1].body, "second");
    assert_eq!(msgs[2].body, "third");
}

#[test]
fn drain_marks_messages_as_drained() {
    let conn = open();
    register_actor(&conn, "s").unwrap();
    let bus = NoOpBusSender;
    send(
        &conn,
        &InboxMessage {
            sender_id: "s".into(),
            recipient_id: "r".into(),
            body: "msg".into(),
        },
        now_ms(),
        &bus,
    )
    .unwrap();

    drain(&conn, "r", now_ms() + 1).unwrap();
    let count = undrained_count(&conn, "r").unwrap();
    assert_eq!(count, 0, "drained messages must not appear in undrained count");
}

#[test]
fn drain_empty_when_no_messages() {
    let conn = open();
    let msgs = drain(&conn, "nobody", now_ms()).unwrap();
    assert!(msgs.is_empty());
}

#[test]
fn drain_caps_at_max_drain_per_turn() {
    let conn = open();
    register_actor(&conn, "s").unwrap();
    let bus = NoOpBusSender;
    for i in 0..MAX_DRAIN_PER_TURN + 10 {
        send(
            &conn,
            &InboxMessage {
                sender_id: "s".into(),
                recipient_id: "r".into(),
                body: format!("msg-{i}"),
            },
            now_ms(),
            &bus,
        )
        .unwrap();
    }

    let msgs = drain(&conn, "r", now_ms() + 1).unwrap();
    assert_eq!(msgs.len(), MAX_DRAIN_PER_TURN);
}

// ── Deferred drain ────────────────────────────────────────────────────────────

#[test]
fn deferred_drain_baseline_zero_on_empty_inbox() {
    let conn = open();
    let baseline = undrained_count(&conn, "agent").unwrap();
    assert_eq!(baseline, 0, "baseline must be 0 when inbox is empty");
}

// ── GC ────────────────────────────────────────────────────────────────────────

#[test]
fn gc_removes_expired_rows() {
    let conn = open();
    register_actor(&conn, "s").unwrap();
    let bus = NoOpBusSender;
    let now = now_ms();
    // Send a message that expires immediately
    send(
        &conn,
        &InboxMessage {
            sender_id: "s".into(),
            recipient_id: "r".into(),
            body: "expiring".into(),
        },
        now,
        &bus,
    )
    .unwrap();

    // GC with now >> expires_at
    let removed = gc(&conn, now + GC_TTL_MS + 1).unwrap();
    assert_eq!(removed, 1);
    let count = undrained_count(&conn, "r").unwrap();
    assert_eq!(count, 0);
}

#[test]
fn gc_keeps_non_expired_rows() {
    let conn = open();
    register_actor(&conn, "s").unwrap();
    let bus = NoOpBusSender;
    let now = now_ms();
    send(
        &conn,
        &InboxMessage {
            sender_id: "s".into(),
            recipient_id: "r".into(),
            body: "fresh".into(),
        },
        now,
        &bus,
    )
    .unwrap();

    // GC at exactly now — expires_at = now + TTL, so nothing removed
    let removed = gc(&conn, now).unwrap();
    assert_eq!(removed, 0);
    let count = undrained_count(&conn, "r").unwrap();
    assert_eq!(count, 1);
}

// ── Constants ─────────────────────────────────────────────────────────────────

#[test]
fn constants_have_expected_values() {
    const { assert!(GC_TTL_MS == 7 * 24 * 60 * 60 * 1_000) }
    const { assert!(MAX_DRAIN_PER_TURN == 100) }
}
