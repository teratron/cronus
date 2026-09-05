//! The catalog half: one registration door for every action's descriptor,
//! the identity rules that keep a contribution from ever shadowing a core
//! name, the manifest-grant gate on registering at all, and the attribution
//! a contributed invocable cannot suppress. See [`super::dispatch`] for the
//! execution half.

use std::collections::{HashMap, HashSet};

use cronus_contract::{Invocable, InvocableId};

/// The reserved identity naming the core itself. No contribution may
/// register under this identity — enforced by pre-claiming it at
/// construction (see [`InvocableRegistry::new`]) rather than as a special
/// case in the registration logic, so the reservation and the general
/// collision rule are the same mechanism.
pub const CORE_IDENTITY: &str = "core";

/// The origin token the core itself registers under.
const CORE_SOURCE: &str = "core";

/// The general capability a contribution's manifest must declare before it
/// may register anything at all (EP-7). A point requiring a further,
/// specific grant for a security-relevant invocable is a real refinement
/// this general gate does not yet model — deferred to whichever task wires
/// up real extension manifests, not silently dropped.
pub const CONTRIBUTE_GRANT: &str = "invocable:contribute";

/// Who is registering, and what physically distinguishes them.
///
/// `identity` is the qualifier a contribution's invocable ids are prefixed
/// with (human-chosen, e.g. an extension's own declared id). `source` is
/// assigned by whatever loads the registrant — never chosen by the identity
/// string alone — so two different origins that happen to pick the same
/// `identity` are still distinguishable, and the registry can tell "the same
/// extension registering more of its own invocables" apart from "two
/// different extensions colliding on one name". `grants` is what the
/// registrant's manifest declared (EP-7) — attaching to this registry at
/// all requires [`CONTRIBUTE_GRANT`] among them; the core registrant needs
/// none, being implicitly trusted (EP-12).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Registrant {
    pub identity: String,
    pub source: String,
    pub grants: HashSet<String>,
}

impl Registrant {
    /// The one core registrant. Its identity is the reserved [`CORE_IDENTITY`].
    pub fn core() -> Self {
        Registrant {
            identity: CORE_IDENTITY.to_string(),
            source: CORE_SOURCE.to_string(),
            grants: HashSet::new(),
        }
    }

    /// A contribution's registrant: its own declared identity, and the
    /// origin token its loader assigned it. Carries no grants yet — chain
    /// [`Registrant::with_grant`] to declare what its manifest grants.
    pub fn extension(identity: impl Into<String>, source: impl Into<String>) -> Self {
        Registrant {
            identity: identity.into(),
            source: source.into(),
            grants: HashSet::new(),
        }
    }

    /// Declare one grant this registrant's manifest carries.
    pub fn with_grant(mut self, grant: impl Into<String>) -> Self {
        self.grants.insert(grant.into());
        self
    }
}

/// Why a registration was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistrationError {
    /// The invocable's own qualified id names a different identity than the
    /// registrant claims to be.
    IdentityMismatch {
        declared: String,
        registrant: String,
    },
    /// This identity is already claimed by a different origin — including
    /// the reserved core identity, pre-claimed by [`CORE_SOURCE`] at
    /// construction, which is what makes claiming `"core"` as anyone else
    /// fail this same check rather than a separate one.
    IdentityCollision { identity: String },
    /// The registrant's manifest did not declare the grant this registry
    /// requires to register anything (EP-7). Never checked for the core
    /// registrant, which needs no declared grant.
    MissingGrant { identity: String, grant: String },
    /// This exact qualified id is already registered.
    DuplicateId(InvocableId),
}

/// One registration door; two lookup paths.
///
/// [`InvocableRegistry::resolve`] finds any registered invocable by its full
/// qualified id — always unambiguous. [`InvocableRegistry::resolve_bare`]
/// finds one by its unqualified tail alone, and only ever resolves to a core
/// invocable: the bare form is the core's reserved projection (EP-11), and a
/// contribution is reachable only by its qualified id, never by colliding
/// with — or ever displacing — a core entry there.
#[derive(Debug, Clone, Default)]
pub struct InvocableRegistry {
    by_id: HashMap<InvocableId, Invocable>,
    bare: HashMap<String, InvocableId>,
    identity_sources: HashMap<String, String>,
}

impl InvocableRegistry {
    /// A registry with the reserved core identity already claimed — no
    /// registration step is required to establish it, and none can revoke
    /// it.
    pub fn new() -> Self {
        let mut identity_sources = HashMap::new();
        identity_sources.insert(CORE_IDENTITY.to_string(), CORE_SOURCE.to_string());
        InvocableRegistry {
            by_id: HashMap::new(),
            bare: HashMap::new(),
            identity_sources,
        }
    }

    /// Register one invocable on behalf of `registrant`. The same function
    /// serves the core and every contribution alike (EP-12) — there is no
    /// separate, privileged registration path.
    pub fn register(
        &mut self,
        registrant: &Registrant,
        invocable: Invocable,
    ) -> Result<(), RegistrationError> {
        let declared = invocable.id.qualifier();
        if declared != registrant.identity {
            return Err(RegistrationError::IdentityMismatch {
                declared: declared.to_string(),
                registrant: registrant.identity.clone(),
            });
        }

        if registrant.identity != CORE_IDENTITY && !registrant.grants.contains(CONTRIBUTE_GRANT) {
            return Err(RegistrationError::MissingGrant {
                identity: registrant.identity.clone(),
                grant: CONTRIBUTE_GRANT.to_string(),
            });
        }

        match self.identity_sources.get(&registrant.identity) {
            Some(existing_source) if existing_source != &registrant.source => {
                return Err(RegistrationError::IdentityCollision {
                    identity: registrant.identity.clone(),
                });
            }
            _ => {
                self.identity_sources
                    .insert(registrant.identity.clone(), registrant.source.clone());
            }
        }

        if self.by_id.contains_key(&invocable.id) {
            return Err(RegistrationError::DuplicateId(invocable.id.clone()));
        }

        if registrant.identity == CORE_IDENTITY {
            self.bare
                .insert(invocable.id.tail().to_string(), invocable.id.clone());
        }

        self.by_id.insert(invocable.id.clone(), invocable);
        Ok(())
    }

    /// Look up by full qualified id — reaches core and contributed
    /// invocables alike.
    pub fn resolve(&self, id: &InvocableId) -> Option<&Invocable> {
        self.by_id.get(id)
    }

    /// Look up by unqualified tail — reaches only a core invocable, per the
    /// bare-form rule (EP-4/EP-11).
    pub fn resolve_bare(&self, tail: &str) -> Option<&Invocable> {
        self.bare.get(tail).and_then(|id| self.by_id.get(id))
    }
}

/// The core-drawn attribution for a contributed invocable, or `None` for a
/// core one (EP-10). Always derived from the id's own qualifier — which
/// [`InvocableRegistry::register`] already validated against the actual
/// registrant before ever accepting it — so nothing the invocable's *other*
/// fields say (`name`, `summary`) can alter or suppress it: the projection
/// draws this independently of whatever the contribution would prefer
/// displayed.
pub fn attribution(invocable: &Invocable) -> Option<&str> {
    let qualifier = invocable.id.qualifier();
    if qualifier == CORE_IDENTITY {
        None
    } else {
        Some(qualifier)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cronus_contract::{Locus, Stability};

    fn sample_invocable(id: &str) -> Invocable {
        Invocable {
            id: InvocableId::from(id),
            name: "Sample",
            summary: "A sample invocable for registry tests.",
            group: "test",
            locus: Locus::Semantic,
            binders: Vec::new(),
            stability: Stability::Shipped,
        }
    }

    fn granted(identity: &str, source: &str) -> Registrant {
        Registrant::extension(identity, source).with_grant(CONTRIBUTE_GRANT)
    }

    #[test]
    fn a_contribution_cannot_claim_the_reserved_core_identity() {
        let mut registry = InvocableRegistry::new();
        // Even granted, an impostor claiming the reserved identity is
        // refused by the identity-collision check, not by the grant gate.
        let impostor = granted(CORE_IDENTITY, "malicious-ext");
        let err = registry
            .register(&impostor, sample_invocable("core:board.list"))
            .unwrap_err();
        assert_eq!(
            err,
            RegistrationError::IdentityCollision {
                identity: CORE_IDENTITY.to_string()
            }
        );
    }

    #[test]
    fn a_bare_verb_collision_resolves_to_the_core_registrant_regardless_of_order() {
        let mut registry = InvocableRegistry::new();
        // The contribution registers FIRST, to prove the outcome does not
        // depend on registration order.
        registry
            .register(
                &granted("myext", "src-1"),
                sample_invocable("myext:board.list"),
            )
            .unwrap();
        registry
            .register(&Registrant::core(), sample_invocable("core:board.list"))
            .unwrap();

        let resolved = registry.resolve_bare("board.list").expect("bare lookup");
        assert_eq!(resolved.id.as_str(), "core:board.list");

        // The contribution stays reachable by its own qualified id.
        assert!(
            registry
                .resolve(&InvocableId::from("myext:board.list"))
                .is_some()
        );
    }

    #[test]
    fn two_sources_claiming_one_identity_the_later_is_refused() {
        let mut registry = InvocableRegistry::new();
        registry
            .register(&granted("myext", "src-1"), sample_invocable("myext:one"))
            .unwrap();

        let err = registry
            .register(&granted("myext", "src-2"), sample_invocable("myext:two"))
            .unwrap_err();
        assert_eq!(
            err,
            RegistrationError::IdentityCollision {
                identity: "myext".to_string()
            }
        );

        // The SAME source registering more under its own identity is fine.
        assert!(
            registry
                .register(&granted("myext", "src-1"), sample_invocable("myext:three"))
                .is_ok()
        );
    }

    #[test]
    fn declared_qualifier_must_match_the_registrants_own_identity() {
        let mut registry = InvocableRegistry::new();
        let err = registry
            .register(
                &granted("myext", "src-1"),
                sample_invocable("otherext:verb"),
            )
            .unwrap_err();
        assert_eq!(
            err,
            RegistrationError::IdentityMismatch {
                declared: "otherext".to_string(),
                registrant: "myext".to_string(),
            }
        );
    }

    #[test]
    fn duplicate_qualified_id_is_refused() {
        let mut registry = InvocableRegistry::new();
        registry
            .register(&Registrant::core(), sample_invocable("core:board.list"))
            .unwrap();
        let err = registry
            .register(&Registrant::core(), sample_invocable("core:board.list"))
            .unwrap_err();
        assert_eq!(
            err,
            RegistrationError::DuplicateId(InvocableId::from("core:board.list"))
        );
    }

    #[test]
    fn registering_without_the_declared_contribute_grant_is_refused() {
        let mut registry = InvocableRegistry::new();
        let ungranted = Registrant::extension("myext", "src-1");
        let err = registry
            .register(&ungranted, sample_invocable("myext:verb"))
            .unwrap_err();
        assert_eq!(
            err,
            RegistrationError::MissingGrant {
                identity: "myext".to_string(),
                grant: CONTRIBUTE_GRANT.to_string(),
            }
        );

        // The same registration, with the grant declared, succeeds.
        assert!(
            registry
                .register(&granted("myext", "src-1"), sample_invocable("myext:verb"))
                .is_ok()
        );
    }

    #[test]
    fn the_core_registrant_needs_no_declared_grant() {
        let mut registry = InvocableRegistry::new();
        assert!(
            registry
                .register(&Registrant::core(), sample_invocable("core:board.list"))
                .is_ok()
        );
    }

    #[test]
    fn attribution_is_derived_from_the_validated_identity_never_from_display_fields() {
        let mut deceptive = sample_invocable("myext:settings");
        // A contribution trying to look like a core feature through the
        // fields it does control.
        deceptive.name = "Core Settings";
        deceptive.summary = "Built-in configuration";
        assert_eq!(attribution(&deceptive), Some("myext"));

        let core_invocable = sample_invocable("core:board.list");
        assert_eq!(attribution(&core_invocable), None);
    }
}
