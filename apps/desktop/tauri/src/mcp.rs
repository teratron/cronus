//! MCP client layer: transports, per-server connection status, OAuth flow.
//!
//! Three transport variants connect a configured server; every remote
//! transport carries the shared default timeout while local stdio has none.
//! Connection state is tracked per server, and the OAuth flow parks the
//! pending transport in a map keyed by server name until the callback
//! resumes it. The wire protocol itself binds behind these types later.

use std::collections::BTreeMap;

/// Timeout for all remote transports; local stdio has none.
pub const DEFAULT_TIMEOUT_MS: u64 = 30_000;

/// How a configured MCP server is reached.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Transport {
    /// Local executable over stdin/stdout — lowest latency, no auth.
    Stdio { command: String },
    /// Hosted endpoint using OAuth-protected Server-Sent Events.
    Sse { url: String, timeout_ms: u64 },
    /// Hosted endpoint using streamable HTTP (MCP HTTP+SSE spec).
    StreamableHttp { url: String, timeout_ms: u64 },
}

impl Transport {
    pub fn stdio(command: impl Into<String>) -> Self {
        Self::Stdio {
            command: command.into(),
        }
    }

    pub fn sse(url: impl Into<String>) -> Self {
        Self::Sse {
            url: url.into(),
            timeout_ms: DEFAULT_TIMEOUT_MS,
        }
    }

    pub fn streamable_http(url: impl Into<String>) -> Self {
        Self::StreamableHttp {
            url: url.into(),
            timeout_ms: DEFAULT_TIMEOUT_MS,
        }
    }

    /// Remote transports time out; local stdio does not.
    pub fn timeout_ms(&self) -> Option<u64> {
        match self {
            Self::Stdio { .. } => None,
            Self::Sse { timeout_ms, .. } | Self::StreamableHttp { timeout_ms, .. } => {
                Some(*timeout_ms)
            }
        }
    }

    /// Whether this transport participates in the OAuth flow.
    pub fn requires_oauth(&self) -> bool {
        !matches!(self, Self::Stdio { .. })
    }
}

/// Connection state of one configured server, as surfaced to diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum McpStatus {
    Connected,
    Disabled,
    Failed { error: String },
    NeedsAuth,
    NeedsClientRegistration { error: String },
}

/// Client capabilities declared at connect. Only `roots` is declared;
/// sampling/elicitation/tasks stay off pending standardization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientCapabilities {
    pub roots: bool,
    pub sampling: bool,
    pub elicitation: bool,
    pub tasks: bool,
}

pub fn client_capabilities() -> ClientCapabilities {
    ClientCapabilities {
        roots: true,
        sampling: false,
        elicitation: false,
        tasks: false,
    }
}

/// The response to `ListRootsRequest`: the current worktree as a file URI.
pub fn roots_response(worktree: &str) -> Vec<String> {
    vec![format!("file:///{}", worktree.trim_start_matches('/'))]
}

/// Outcome of an OAuth callback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OauthOutcome {
    Success,
    /// Retryable failure — the user can restart the flow.
    AuthFailed,
    /// The server demands dynamic client registration first.
    RegistrationRequired {
        error: String,
    },
}

/// Per-server registry: connection status plus the pending OAuth transports.
#[derive(Debug, Default)]
pub struct McpRegistry {
    status: BTreeMap<String, McpStatus>,
    pending_oauth_transports: BTreeMap<String, Transport>,
}

impl McpRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a server's status (configured, connected, failed, …).
    pub fn set_status(&mut self, server: impl Into<String>, status: McpStatus) {
        self.status.insert(server.into(), status);
    }

    /// The status surfaced to diagnostics; unconfigured servers have none.
    pub fn status(&self, server: &str) -> Option<&McpStatus> {
        self.status.get(server)
    }

    /// Start the OAuth flow: park the pending transport and mark the server
    /// as needing auth (a browser open is attempted by the caller).
    pub fn begin_oauth(&mut self, server: impl Into<String>, transport: Transport) {
        let server = server.into();
        self.pending_oauth_transports
            .insert(server.clone(), transport);
        self.status.insert(server, McpStatus::NeedsAuth);
    }

    /// Whether a server has a parked transport awaiting the callback.
    pub fn oauth_pending(&self, server: &str) -> bool {
        self.pending_oauth_transports.contains_key(server)
    }

    /// Complete the OAuth flow for a server: resume the parked transport on
    /// success, or record the failure class. The pending entry is always
    /// consumed — a retry restarts the flow from `begin_oauth`.
    pub fn complete_oauth(&mut self, server: &str, outcome: OauthOutcome) -> Option<Transport> {
        let transport = self.pending_oauth_transports.remove(server);
        let status = match outcome {
            OauthOutcome::Success => McpStatus::Connected,
            OauthOutcome::AuthFailed => McpStatus::NeedsAuth,
            OauthOutcome::RegistrationRequired { error } => {
                McpStatus::NeedsClientRegistration { error }
            }
        };
        self.status.insert(server.to_string(), status);
        match outcome_is_success(self.status.get(server)) {
            true => transport,
            false => None,
        }
    }
}

fn outcome_is_success(status: Option<&McpStatus>) -> bool {
    matches!(status, Some(McpStatus::Connected))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transports_construct_with_the_default_timeout() {
        assert_eq!(Transport::stdio("cronus-mcp").timeout_ms(), None);
        assert_eq!(
            Transport::sse("https://mcp.example/sse").timeout_ms(),
            Some(DEFAULT_TIMEOUT_MS)
        );
        assert_eq!(
            Transport::streamable_http("https://mcp.example").timeout_ms(),
            Some(DEFAULT_TIMEOUT_MS)
        );
        assert!(!Transport::stdio("x").requires_oauth());
        assert!(Transport::sse("u").requires_oauth());
        assert!(Transport::streamable_http("u").requires_oauth());
    }

    #[test]
    fn status_machine_covers_all_five_states() {
        let mut registry = McpRegistry::new();
        registry.set_status("srv", McpStatus::Disabled);
        assert_eq!(registry.status("srv"), Some(&McpStatus::Disabled));

        registry.set_status(
            "srv",
            McpStatus::Failed {
                error: "connection refused".into(),
            },
        );
        assert!(matches!(
            registry.status("srv"),
            Some(McpStatus::Failed { .. })
        ));

        registry.begin_oauth("srv", Transport::sse("https://mcp.example/sse"));
        assert_eq!(registry.status("srv"), Some(&McpStatus::NeedsAuth));

        registry.complete_oauth(
            "srv",
            OauthOutcome::RegistrationRequired {
                error: "dynamic registration required".into(),
            },
        );
        assert!(matches!(
            registry.status("srv"),
            Some(McpStatus::NeedsClientRegistration { .. })
        ));

        registry.begin_oauth("srv", Transport::sse("https://mcp.example/sse"));
        registry.complete_oauth("srv", OauthOutcome::Success);
        assert_eq!(registry.status("srv"), Some(&McpStatus::Connected));
    }

    #[test]
    fn oauth_flow_tracks_the_pending_transport_map() {
        let mut registry = McpRegistry::new();
        let transport = Transport::streamable_http("https://mcp.example");

        registry.begin_oauth("srv", transport.clone());
        assert!(registry.oauth_pending("srv"));

        let resumed = registry.complete_oauth("srv", OauthOutcome::Success);
        assert_eq!(resumed, Some(transport), "success resumes the transport");
        assert!(!registry.oauth_pending("srv"), "entry consumed");

        // A failed callback also consumes the entry (retry restarts the flow).
        registry.begin_oauth("srv2", Transport::sse("https://two.example"));
        let resumed = registry.complete_oauth("srv2", OauthOutcome::AuthFailed);
        assert_eq!(resumed, None);
        assert!(!registry.oauth_pending("srv2"));
        assert_eq!(registry.status("srv2"), Some(&McpStatus::NeedsAuth));
    }

    #[test]
    fn client_declares_roots_only_and_answers_with_the_worktree() {
        let caps = client_capabilities();
        assert!(caps.roots);
        assert!(!caps.sampling && !caps.elicitation && !caps.tasks);

        let roots = roots_response("/path/to/worktree");
        assert_eq!(roots, vec!["file:///path/to/worktree".to_string()]);
    }
}
