//! `cronus-model-local` — the streaming REST transport realizing
//! `l2-model-runtime`: endpoint profiles over the federated local provider
//! catalog (technology-stack §4.4), plus (in later phase tasks) the
//! streaming generate call, embed/describe/pull, and failure mapping.
//!
//! This module's current scope (T-17B01): the endpoint-profile model and
//! its reachability probe. The *address* (`api_base`) always comes from the
//! caller (the router's policy, `l2-model-router`) — this crate never
//! invents or looks one up; it only adds the how-to-talk layer (protocol
//! family, capability flags, probe rules) over that address.

use std::io;
use std::net::{IpAddr, SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;

/// The stack §4.4 probe discipline: no probe blocks longer than this by
/// default. Tests override via `probe_with_timeout`.
pub const DEFAULT_PROBE_TIMEOUT: Duration = Duration::from_millis(800);

/// Which request/response shape a provider speaks (§4.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolFamily {
    /// OpenAI-compatible `/v1/...` surface (llama.cpp, MLX, vLLM, LM Studio,
    /// Docker Model Runner).
    OpenAiCompatible,
    /// Provider-native REST (e.g. Ollama's `/api/*`).
    Native,
}

/// Capability flags an endpoint declares (MR-2/MR-6/MR-9) — data, not a
/// promise: a capability absent here is reported to the caller as absent,
/// never silently emulated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Capabilities {
    pub streaming: bool,
    pub embeddings: bool,
    pub pull: bool,
    pub residency_control: bool,
    pub digest_reporting: bool,
}

/// Why constructing an `EndpointProfile` was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileError {
    /// `api_base` did not parse into a connectable host[:port].
    InvalidAddress(String),
    /// The host is not loopback, and `EndpointProfile::new` is loopback-only
    /// by default (MR-1) — a remote endpoint needs the (egress-gated)
    /// remote constructor landing in a later task.
    NotLoopback(String),
    /// The host is a wildcard bind address (`0.0.0.0` / `::`), never a
    /// valid connect target (stack §4.4).
    WildcardAddress(String),
}

/// Outcome of a reachability probe (§4.3, mirrors technology-stack §4.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeOutcome {
    Reachable,
    ConnectionRefused,
    Timeout,
    InvalidAddress,
}

/// How to talk to one catalog provider (§4.3) — data, not code. The address
/// (`api_base`) is supplied by the caller (the router's policy); this
/// profile adds only the how-to-talk layer, never a parallel address
/// registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndpointProfile {
    api_base: String,
    protocol: ProtocolFamily,
    capabilities: Capabilities,
}

impl EndpointProfile {
    /// Construct a loopback-only profile — the default per MR-1. Rejects a
    /// non-loopback host and a wildcard bind address; a remote profile is a
    /// distinct, egress-gated construction path (later task).
    pub fn new(
        api_base: impl Into<String>,
        protocol: ProtocolFamily,
        capabilities: Capabilities,
    ) -> Result<Self, ProfileError> {
        let api_base = api_base.into();
        let host =
            host_of(&api_base).ok_or_else(|| ProfileError::InvalidAddress(api_base.clone()))?;

        if !host.eq_ignore_ascii_case("localhost") {
            match host.parse::<IpAddr>() {
                Ok(ip) if ip.is_unspecified() => {
                    return Err(ProfileError::WildcardAddress(api_base));
                }
                Ok(ip) if ip.is_loopback() => {}
                _ => return Err(ProfileError::NotLoopback(api_base)),
            }
        }

        Ok(EndpointProfile {
            api_base,
            protocol,
            capabilities,
        })
    }

    pub fn api_base(&self) -> &str {
        &self.api_base
    }

    pub fn protocol(&self) -> ProtocolFamily {
        self.protocol
    }

    pub fn capabilities(&self) -> Capabilities {
        self.capabilities
    }

    /// Reachability probe with the stack's default 800ms budget.
    pub fn probe(&self) -> ProbeOutcome {
        self.probe_with_timeout(DEFAULT_PROBE_TIMEOUT)
    }

    /// Reachability probe with an explicit timeout (test seam) — a pure TCP
    /// connectivity check, no wire protocol spoken.
    pub fn probe_with_timeout(&self, timeout: Duration) -> ProbeOutcome {
        let Some(addr) = resolve_one(&self.api_base) else {
            return ProbeOutcome::InvalidAddress;
        };
        match TcpStream::connect_timeout(&addr, timeout) {
            Ok(_) => ProbeOutcome::Reachable,
            Err(e) if e.kind() == io::ErrorKind::ConnectionRefused => {
                ProbeOutcome::ConnectionRefused
            }
            Err(e) if e.kind() == io::ErrorKind::TimedOut => ProbeOutcome::Timeout,
            // A conservative default: any other connect failure (e.g. "no
            // route to host", host-OS-dependent) is reported as Timeout —
            // "not reachable within budget" is the honest characterization
            // shared by every non-refused failure mode.
            Err(_) => ProbeOutcome::Timeout,
        }
    }
}

/// Strip an optional `scheme://` prefix and any path/query suffix, leaving
/// just the `host[:port]` (or bracketed-IPv6 `[host]:port`) portion.
fn strip_scheme_and_path(api_base: &str) -> &str {
    let without_scheme = api_base
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(api_base);
    without_scheme.split('/').next().unwrap_or(without_scheme)
}

/// Extract just the host (no port, no brackets) from an `api_base` string.
fn host_of(api_base: &str) -> Option<String> {
    let host_port = strip_scheme_and_path(api_base);
    if let Some(rest) = host_port.strip_prefix('[') {
        // Bracketed IPv6: "[::1]:8080" or "[::1]".
        let host = rest.split(']').next()?;
        return if host.is_empty() {
            None
        } else {
            Some(host.to_string())
        };
    }
    // "host:port" or bare "host" — a literal IPv4/hostname has at most one
    // colon (the port separator), so splitting on the last colon is safe.
    let host = host_port
        .rsplit_once(':')
        .map(|(h, _)| h)
        .unwrap_or(host_port);
    if host.is_empty() {
        None
    } else {
        Some(host.to_string())
    }
}

/// Resolve `api_base` to one connectable `SocketAddr`.
fn resolve_one(api_base: &str) -> Option<SocketAddr> {
    let host_port = strip_scheme_and_path(api_base);
    host_port.to_socket_addrs().ok()?.next()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    fn caps() -> Capabilities {
        Capabilities {
            streaming: true,
            ..Default::default()
        }
    }

    #[test]
    fn accepts_the_real_default_localhost_endpoints() {
        // The stack §4.4 catalog's actual defaults use the literal
        // hostname "localhost", not a numeric loopback IP.
        for api_base in [
            "http://localhost:11434", // Ollama
            "http://localhost:8080",  // llama.cpp / MLX
            "http://localhost:8000",  // vLLM
            "http://localhost:1234",  // LM Studio
            "http://localhost:12434", // Docker Model Runner
        ] {
            let profile = EndpointProfile::new(api_base, ProtocolFamily::OpenAiCompatible, caps())
                .unwrap_or_else(|e| panic!("{api_base} should be accepted, got {e:?}"));
            assert_eq!(profile.api_base(), api_base);
        }
    }

    #[test]
    fn accepts_numeric_loopback_hosts() {
        assert!(
            EndpointProfile::new("http://127.0.0.1:8080", ProtocolFamily::Native, caps()).is_ok()
        );
        assert!(EndpointProfile::new("http://[::1]:8080", ProtocolFamily::Native, caps()).is_ok());
    }

    #[test]
    fn rejects_wildcard_bind_addresses() {
        assert_eq!(
            EndpointProfile::new("http://0.0.0.0:8080", ProtocolFamily::Native, caps()),
            Err(ProfileError::WildcardAddress(
                "http://0.0.0.0:8080".to_string()
            ))
        );
        assert_eq!(
            EndpointProfile::new("http://[::]:8080", ProtocolFamily::Native, caps()),
            Err(ProfileError::WildcardAddress(
                "http://[::]:8080".to_string()
            ))
        );
    }

    #[test]
    fn rejects_non_loopback_hosts_by_default() {
        assert_eq!(
            EndpointProfile::new("http://192.168.1.5:8080", ProtocolFamily::Native, caps()),
            Err(ProfileError::NotLoopback(
                "http://192.168.1.5:8080".to_string()
            ))
        );
        assert_eq!(
            EndpointProfile::new("http://example.com:8080", ProtocolFamily::Native, caps()),
            Err(ProfileError::NotLoopback(
                "http://example.com:8080".to_string()
            ))
        );
    }

    #[test]
    fn probe_classifies_reachable_against_a_real_listener() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind an ephemeral port");
        let port = listener.local_addr().expect("local addr").port();
        let profile = EndpointProfile::new(
            format!("http://127.0.0.1:{port}"),
            ProtocolFamily::Native,
            caps(),
        )
        .expect("loopback profile");

        assert_eq!(profile.probe(), ProbeOutcome::Reachable);
    }

    #[test]
    fn probe_classifies_a_closed_port_as_definitively_unreachable() {
        // Bind to let the OS assign a free port, then drop the listener —
        // the port is now provably closed on this host.
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind an ephemeral port");
        let port = listener.local_addr().expect("local addr").port();
        drop(listener);

        let profile = EndpointProfile::new(
            format!("http://127.0.0.1:{port}"),
            ProtocolFamily::Native,
            caps(),
        )
        .expect("loopback profile");

        // A closed port normally resets the connection immediately
        // (`ConnectionRefused`). Some sandboxed/virtualized network layers
        // (confirmed empirically for this execution environment) instead
        // suppress the RST and let the connect attempt run out the clock,
        // which this probe honestly reports as `Timeout` rather than
        // fabricating a refusal it never observed (see the `Err(_) =>
        // Timeout` fallback above) — both outcomes agree the endpoint is
        // not serving, which is what this test asserts.
        let outcome = profile.probe_with_timeout(Duration::from_millis(300));
        assert!(
            matches!(
                outcome,
                ProbeOutcome::ConnectionRefused | ProbeOutcome::Timeout
            ),
            "a closed port must classify as refused or (honestly) timed-out, got {outcome:?}"
        );
    }

    #[test]
    fn probe_classifies_timeout_within_the_given_budget() {
        // RFC 5737 TEST-NET-1: reserved for documentation, never routed —
        // a deterministic, hermetic non-responder. Uses a short timeout
        // (not the 800ms default) to keep the test fast.
        let profile = EndpointProfile {
            api_base: "192.0.2.1:80".to_string(),
            protocol: ProtocolFamily::Native,
            capabilities: caps(),
        };

        let start = std::time::Instant::now();
        let outcome = profile.probe_with_timeout(Duration::from_millis(300));
        let elapsed = start.elapsed();

        assert_eq!(outcome, ProbeOutcome::Timeout);
        assert!(
            elapsed < Duration::from_millis(800),
            "probe must honor the given budget, took {elapsed:?}"
        );
    }
}
