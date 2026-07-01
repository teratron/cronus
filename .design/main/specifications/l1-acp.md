# Agent Client Protocol (ACP)

**Version:** 1.1.0
**Status:** Stable
**Layer:** concept

## Overview

The Agent Client Protocol (ACP) is the abstract communication protocol through which external callers — other agents, user-facing clients, CI/CD pipelines, remote offices — interact with a running Cronus office. ACP defines the session lifecycle, message framing, capability discovery, streaming response contract, and tool delegation boundary. It is transport-agnostic: a concrete transport (HTTP/SSE, WebSocket, Unix socket) is an L2 implementation concern; the protocol semantics are defined here at L1.

ACP is the external-facing complement to the internal orchestration protocol: orchestration governs agent-to-agent coordination within an office; ACP governs client-to-office communication across the office boundary.

## Related Specifications

- [l1-orchestration.md](l1-orchestration.md) — orchestration protocol that ACP sessions invoke on the inside
- [l1-security.md](l1-security.md) — authentication and authorization gates that ACP sessions pass through
- [l1-office-model.md](l1-office-model.md) — the office entity that hosts the ACP server endpoint
- [l2-agent-session.md](l2-agent-session.md) — concrete daemon server implementation (`/acp` endpoint, Streamable HTTP transport)
- [l1-global-orchestration.md](l1-global-orchestration.md) — cross-office ACP routing at the building level

## 1. Motivation

Autonomous offices need external entry points: a remote user, another office, a CI job, or a mobile client must be able to start and continue an agent interaction without being co-located. Without a defined protocol:
- Clients are tightly coupled to implementation details that change between versions
- Security boundaries are implicit and inconsistently enforced
- Streaming responses (agent thinking, tool calls, partial text) have no canonical format
- Capability discovery requires reading source code rather than querying the running server

ACP defines these boundaries explicitly, enabling any conformant client to connect to any conformant Cronus office.

## 2. Constraints & Assumptions

- ACP is a request-response and streaming protocol. A single client message may produce a stream of events before a terminal response.
- The protocol is stateful at the session level: a session retains context across multiple turns. The office retains session state; the client need not.
- ACP does not define the serialization format (JSON, MessagePack, protobuf) — that is an L2 transport decision.
- ACP does not define authentication mechanisms (Bearer token, API key, mTLS) — those are enforced at the transport layer per `l1-security.md`.
- A client is trusted or untrusted based on its authentication identity; an untrusted client receives a restricted capability set.

## 3. Core Invariants

- **ACP-1 Session-oriented**: every ACP interaction occurs within a named session. A session maintains context (conversation history, in-progress task state, token budget) across turns. Stateless request-response without a session identifier is not ACP.
- **ACP-2 Capability declaration**: before any task interaction, the server MUST respond to a `capabilities` query with a machine-readable description of what the current office can do, including: available tool categories, active roles, budget limits, and any restrictions imposed by the trust level of the caller.
- **ACP-3 Streaming event protocol**: multi-step responses (thinking, tool-call, intermediate result, final text) are delivered as a stream of typed events, not as a single terminal message. The client may consume events incrementally; the stream MUST terminate with a terminal event (`done` or `error`).
- **ACP-4 Tool delegation boundary**: the office MAY delegate tool calls to the client on behalf of the agent (client-side tools, browser interactions, local file access). The client MUST explicitly opt in to tool delegation; the server MUST NOT delegate tools to a client that has not opted in.
- **ACP-5 Idempotent session creation**: creating a session with an existing session ID is idempotent — the server returns the existing session's state rather than creating a duplicate. Clients MAY safely retry session creation after network errors.
- **ACP-6 Graceful interrupt**: a client MAY send an `interrupt` signal at any time during a streaming response. The server MUST honour the interrupt, complete any currently-running atomic step (ORC-5 context isolation), then return a partial terminal event. The interrupted session remains valid and resumable.
- **ACP-7 Budget transparency**: the office MUST report the remaining token/cost budget for the current session in every terminal event. A client that receives a `budget_exhausted` terminal event MUST NOT retry immediately — the office is in hibernation (see `l1-office-control.md` OC-4).

- **ACP-8 Monotonic event ordering** [ADDED v1.1.0]: every streamed event carries a **per-session monotonic sequence number** assigned in emission order. A client orders and deduplicates the event stream by sequence number, not by arrival order, and MUST tolerate a transport that reorders or redelivers. The terminal event carries the highest sequence number of its turn; a gap in the sequence signals dropped events and MUST be surfaced, not silently ignored. This makes stream ordering an explicit protocol contract rather than a property of a particular transport's delivery guarantees.

- **ACP-9 Protocol projections are pure adapters** [ADDED v1.1.0]: one running session MAY be exposed simultaneously through several external protocol surfaces (e.g. a peer-agent protocol, a client-completion protocol, a UI-event protocol). Each surface is a **pure, logic-free adapter** over the single ACP event stream: it re-frames the same ordered events (ACP-8) into its own wire shape and MUST NOT add, drop, reorder, or reinterpret semantics, hold business logic, or fork session state. All surfaces bound to a session observe the identical event sequence. Adding a protocol is adding an adapter, never changing the office.

## 4. Detailed Design

### 4.1 Session Lifecycle

```text
[REFERENCE]
Client                                    ACP Server (Office)
  │                                             │
  │── create_session(capabilities_offer) ──────►│  (ACP-1, ACP-5)
  │◄─ session_id + server_capabilities ─────────│  (ACP-2)
  │                                             │
  │── send_message(session_id, content) ────────►│
  │◄─ stream: [thinking?, tool_call*, text_delta*, done] ─│  (ACP-3)
  │                                             │
  │── send_message(...) ────────────────────────►│  (multi-turn)
  │◄─ stream: [...]  ────────────────────────────│
  │                                             │
  │── interrupt(session_id) ────────────────────►│  (ACP-6)
  │◄─ partial_done + checkpoint_id ─────────────│
  │                                             │
  │── close_session(session_id) ────────────────►│
  │◄─ ack  ──────────────────────────────────────│
```

### 4.2 Event Types (Streaming)

| Event type | Meaning | Terminal? |
| --- | --- | --- |
| `thinking` | Agent internal reasoning step (if exposed) | No |
| `tool_call` | Agent is invoking a server-side tool; client may observe | No |
| `client_tool_request` | Office is delegating a tool call to the client (ACP-4) | No |
| `client_tool_response` | Client provides the result of a delegated tool call | No |
| `text_delta` | Partial text content (streaming output) | No |
| `done` | Session turn complete; includes final text + remaining budget | Yes |
| `error` | Unrecoverable error for this turn; session remains valid | Yes |
| `budget_exhausted` | Token/cost budget consumed; office entering hibernation | Yes |
| `interrupted` | Turn interrupted by client signal (ACP-6); session resumable | Yes |

### 4.3 Capability Declaration

The `capabilities` response describes the office's current abilities as a machine-readable document. It includes:

| Field | Content |
| --- | --- |
| `office_id` | Unique office identifier |
| `version` | ACP protocol version (e.g., `"1.0"`) |
| `tools` | Categories of tools available to the session (read-only, write, code execution, etc.) |
| `roles` | Active worker roles and their declared specialties |
| `trust_level` | `trusted` / `restricted` — the effective trust level for this caller |
| `budget` | Remaining token/cost budget for this session |
| `session_mode` | `interactive` / `autonomous` / `batch` — how the office interprets messages |
| `streaming` | `true` — office always streams; clients that want batched responses must buffer |

### 4.4 Trust Levels

| Trust level | Caller type | Tool access | Budget access | Admin ops |
| --- | --- | --- | --- | --- |
| `trusted` | Authenticated local user, admin token | Full | Read + modify | Yes |
| `restricted` | Authenticated external caller (other office, CI) | Declared subset | Read-only | No |
| `anonymous` | Unauthenticated caller | None | None | No |

Anonymous callers receive only `capabilities` responses. All other operations return `AUTH_REQUIRED`.

### 4.5 Cross-Office Routing

When a caller address targets a different office within the same building, the receiving office's ACP server acts as a **transparent relay**: it forwards the message to the target office and streams the response back. The relay does not inspect or modify the message content. This is the ACP-level mechanism behind the global orchestration routing (see `l1-global-orchestration.md`).

### 4.6 Protocol Projections [ADDED v1.1.0]

ACP is the office's *canonical* external contract (ACP-8 ordered event stream). A
**protocol projection** publishes that one running session under a foreign
protocol without duplicating any office logic:

```text
[REFERENCE]
                         ┌─ projection: peer-agent protocol ──►  peer offices / agents
one ACP session ──stream─┼─ projection: client-completion API ─►  generic LLM-style clients
 (ordered events, ACP-8) ├─ projection: UI-event protocol ─────►  interactive front-ends
                         └─ projection: (future) …

adapter contract (ACP-9):
  bind(session) -> foreign_endpoint
  on each ACP event e:  emit  translate_shape(e)      // pure re-framing, no logic
  invariants: same order, same set, no added meaning, no forked state
```

Each projection is a thin binding: it maps ACP event shapes to the foreign wire
format and back, nothing more. Because every projection consumes the *same*
ordered stream, two clients on two different protocols observing one session see
the same turns in the same order. Capability declaration (ACP-2) and trust levels
(§4.4) are enforced once at the session, beneath all projections — an adapter can
never widen a caller's capabilities. This lets a single office answer peer agents,
generic completion clients, and UI front-ends concurrently, each in its native
protocol, with zero logic duplication and one source of truth for session state.

## 5. Implementation Notes

1. The concrete ACP transport for the daemon server is Streamable HTTP (`POST /acp`, Server-Sent Events for streaming) per `l2-agent-session.md`.
2. `create_session` idempotency (ACP-5) is enforced by a unique-index on `session_id` in the daemon's session store; concurrent creation races return the winner's state to both callers.
3. The `thinking` event is optional — emitted only when the office's `thinking_level` is set above `off`.

## 6. Drawbacks & Alternatives

**Alternative: REST-only, no streaming** — simpler client implementation, but multi-step responses (tool calls, long generation) require polling. Polling is fundamentally incompatible with real-time chat interaction. Rejected.

**Alternative: WebSocket-first** — bidirectional streaming, but requires stateful connection management (reconnect, backpressure). SSE over HTTP is simpler, unidirectional, and sufficient for the client-to-office pattern. Transport is an L2 decision; ACP does not prescribe it.

**Alternative: no session concept — stateless per-request** — simplest protocol, but the orchestrator needs conversation history and task state to function autonomously. Stateless requests force the client to re-send full context on every call, wasting tokens and breaking mid-task interruption/resume. Rejected.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[ORCHESTRATION]` | `.design/main/specifications/l1-orchestration.md` | Internal coordination protocol invoked by ACP sessions |
| `[SECURITY]` | `.design/main/specifications/l1-security.md` | Authentication and authorization gates |
| `[AGENT-SESSION]` | `.design/main/specifications/l2-agent-session.md` | Concrete ACP daemon server (Streamable HTTP `/acp`) |
| `[GLOBAL-ORCH]` | `.design/main/specifications/l1-global-orchestration.md` | Cross-office routing at building level |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.1.0 | 2026-07-01 | Core Team | Added ACP-8 (per-session monotonic event ordering — order/dedup by sequence, transport-reorder-tolerant, gap = dropped events) and ACP-9 (protocol projections are pure logic-free adapters; N surfaces over one session observe the identical event sequence); new §4.6 Protocol Projections (one session projected under peer-agent / client-completion / UI-event protocols concurrently, capabilities+trust enforced once beneath all projections). |
| 1.0.0 | 2026-06-24 | Core Team | Initial spec — ACP-1…ACP-7, session lifecycle, streaming event taxonomy, capability declaration, trust levels, cross-office routing |
