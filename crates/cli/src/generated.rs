//! The semantic half of the command surface (the split between what acts on
//! the user's work and what configures the product itself, §4.1.1): a
//! `clap::Command` tree built at startup from the registry's `Semantic`,
//! `Shipped` descriptors — never a compile-time enum. Help, completion, and
//! grouping for this half all derive from the same descriptors; there is
//! nothing here to restate by hand, and a verb an extension contributes
//! becomes reachable the moment it registers, without a rebuild.
//!
//! The installation half (verbs that configure the product itself rather
//! than act on the user's work) stays a hand-declared, compile-time `clap`
//! grammar — a launcher must be answerable before this generated half can
//! even exist, since it is built from a registry that only comes alive
//! after composition.
//!
//! # Positional-only, except `Flag`
//!
//! Every `Text`/`Integer`/`Boolean` binder becomes a positional argument,
//! bound in declared order. The project's own CLI grammar also uses named
//! `--property` flags (`set <id> --actor <name>`), but `Binder` has no
//! positional-vs-named-flag distinction yet — every migrated verb so far
//! (the `memory` group) only ever needed positionals, so this gap is real
//! but unforced. A verb whose old shape used a named *value* flag
//! (`--actor cli`) is not migrated by this pass; resolving the distinction
//! is follow-up work, not invented here under the pressure of one group
//! that does not need it.
//!
//! `BinderKind::Flag` is the one exception, bound as `--name` rather than
//! positionally — a flag typed as a bare positional word ("type `presets`
//! to enable it") is not a shape any real CLI flag takes, so this one kind
//! does not wait on the general positional-vs-flag design question.

use std::collections::{HashMap, HashSet};

use clap::{Arg, ArgAction, ArgMatches, Command};
use cronus_contract::{ArgValue, ArgValues, Binder, BinderKind, Invocable, Locus, Stability};
use cronus_core::invocable::InvocableRegistry;

/// Every descriptor this generated half is responsible for: reachable from
/// every surface (`Semantic`) and currently on the default listing
/// (`Shipped`) — a `Retired` descriptor keeps resolving (INV-9) but does
/// not appear here, and neither does anything `ClientLocal`/`HostOnly`/
/// `Installation`, which this frontend does not project at all or projects
/// through its own hand-declared grammar instead.
pub fn semantic_shipped(registry: &InvocableRegistry) -> Vec<&Invocable> {
    let mut invocables: Vec<&Invocable> = registry
        .all()
        .filter(|invocable| {
            matches!(invocable.locus, Locus::Semantic)
                && matches!(invocable.stability, Stability::Shipped)
        })
        .collect();
    // Deterministic order (EP-4): grouped, then alphabetical within a
    // group, so `--help` output and iteration order never depend on
    // registration order.
    invocables.sort_by(|a, b| (a.group, a.id.as_str()).cmp(&(b.group, b.id.as_str())));
    invocables
}

/// The verb a descriptor exposes within its own group: its id's tail with
/// the `"<group>."` prefix stripped. `invocable.group` is the declared
/// field, not re-derived from the id — the tail is only ever split to
/// recover the part after it.
///
/// `pub(crate)`: the installation half's own tree builder shares this exact
/// rule (a flat, single-verb group's one verb has a tail equal to its own
/// group name) rather than restating it.
pub(crate) fn verb_of(invocable: &Invocable) -> &str {
    let tail = invocable.id.tail();
    tail.strip_prefix(invocable.group)
        .and_then(|rest| rest.strip_prefix('.'))
        .unwrap_or(tail)
}

/// Build one `clap::Arg` for `binder`, bound positionally in declared order
/// (see the module doc on why every binder is positional for now).
///
/// `pub(crate)`: the installation half's own tree builder binds the same
/// `Binder` kinds the same way — one mapping, not two.
pub(crate) fn arg_for(binder: &Binder) -> Arg {
    let arg = Arg::new(binder.name);
    match binder.kind {
        BinderKind::Text => arg.required(!binder.optional),
        BinderKind::Integer => arg
            .required(!binder.optional)
            .value_parser(clap::value_parser!(i64)),
        BinderKind::Boolean => arg
            .required(!binder.optional)
            .value_parser(clap::value_parser!(bool)),
        // A flag is never positional — "type the word `presets` to enable
        // it" is not a shape any real CLI flag takes — so this is the one
        // binder kind bound as `--name` rather than by position, and it is
        // never required (its absence simply means false).
        BinderKind::Flag => arg
            .long(binder.name)
            .required(false)
            .action(ArgAction::SetTrue),
        // A named text value — `--actor cli`, `--mode login` — bound the
        // same way `Flag` already is (by its own name, not by position),
        // but carrying a value rather than a presence bit.
        BinderKind::NamedText => arg.long(binder.name).required(!binder.optional),
    }
}

/// Build the clap subcommand tree for the semantic half: one top-level
/// subcommand per declared `group`, one verb-level subcommand per
/// descriptor within it, its positional args built from declared binders.
/// Returns the tree alongside the set of group names it owns, so a caller
/// can tell a registry-generated group apart from one still on the old
/// hand-declared enum without re-deriving the same set twice.
pub fn build_semantic_tree(invocables: &[&Invocable]) -> (Vec<Command>, HashSet<String>) {
    let mut by_group: HashMap<&str, Vec<&Invocable>> = HashMap::new();
    for invocable in invocables {
        by_group.entry(invocable.group).or_default().push(invocable);
    }

    let mut group_names = HashSet::new();
    let mut groups = Vec::new();
    let mut group_order: Vec<&str> = by_group.keys().copied().collect();
    group_order.sort_unstable();

    for group in group_order {
        let members = &by_group[group];
        let mut verb_command = Command::new(group.to_string()).about(format!("{group} operations"));
        for invocable in members.iter() {
            let mut verb = Command::new(verb_of(invocable).to_string()).about(invocable.summary);
            for binder in &invocable.binders {
                verb = verb.arg(arg_for(binder));
            }
            verb_command = verb_command.subcommand(verb);
        }
        group_names.insert(group.to_string());
        groups.push(verb_command);
    }

    (groups, group_names)
}

/// Whether `matches`' matched top-level subcommand names a registry-
/// generated group — the caller's cue to route through the registry
/// instead of falling back to the hand-declared enum.
pub fn matched_generated_group<'a>(
    matches: &'a ArgMatches,
    group_names: &HashSet<String>,
) -> Option<(&'a str, &'a ArgMatches)> {
    let (name, sub_matches) = matches.subcommand()?;
    if group_names.contains(name) {
        Some((name, sub_matches))
    } else {
        None
    }
}

/// Resolve the matched group's verb subcommand against `invocables`,
/// returning the invocable it names and the bound arguments — `None` when
/// the group matched but no verb subcommand was actually given (clap's own
/// `--help`/usage handling covers that case before this is ever reached in
/// practice).
pub fn invocation_from_matches<'a>(
    group_matches: &ArgMatches,
    invocables: &'a [&'a Invocable],
) -> Option<(&'a Invocable, ArgValues)> {
    let (verb_name, verb_matches) = group_matches.subcommand()?;
    let invocable = invocables
        .iter()
        .find(|invocable| verb_of(invocable) == verb_name)?;

    let mut args = ArgValues::new();
    for binder in &invocable.binders {
        let value = match binder.kind {
            BinderKind::Text => verb_matches
                .get_one::<String>(binder.name)
                .map(|value| ArgValue::Text(value.clone())),
            BinderKind::Integer => verb_matches
                .get_one::<i64>(binder.name)
                .map(|value| ArgValue::Integer(*value)),
            BinderKind::Boolean => verb_matches
                .get_one::<bool>(binder.name)
                .map(|value| ArgValue::Boolean(*value)),
            BinderKind::Flag => verb_matches.get_flag(binder.name).then_some(ArgValue::Flag),
            BinderKind::NamedText => verb_matches
                .get_one::<String>(binder.name)
                .map(|value| ArgValue::Text(value.clone())),
        };
        if let Some(value) = value {
            args.insert(binder.name, value);
        }
    }
    Some((invocable, args))
}

#[cfg(test)]
mod tests {
    use cronus_contract::{Invocable, InvocableId, Stability};

    use super::*;

    fn descriptor(id: &str, group: &'static str, binders: Vec<Binder>) -> Invocable {
        Invocable {
            id: InvocableId::new(id).expect("well-formed test id"),
            name: "Test",
            summary: "A test descriptor.",
            group,
            locus: Locus::Semantic,
            binders,
            stability: Stability::Shipped,
            journal_raw_input: true,
        }
    }

    #[test]
    fn verb_of_strips_the_group_prefix() {
        let d = descriptor("core:memory.store", "memory", Vec::new());
        assert_eq!(verb_of(&d), "store");
    }

    #[test]
    fn build_semantic_tree_groups_by_declared_group_and_reports_names() {
        let a = descriptor(
            "core:memory.store",
            "memory",
            vec![Binder {
                name: "key",
                kind: BinderKind::Text,
                optional: false,
            }],
        );
        let b = descriptor("core:memory.search", "memory", Vec::new());
        let invocables = vec![&a, &b];

        let (groups, names) = build_semantic_tree(&invocables);
        assert_eq!(names, HashSet::from(["memory".to_string()]));
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].get_name(), "memory");
        let verb_names: Vec<&str> = groups[0].get_subcommands().map(|c| c.get_name()).collect();
        assert_eq!(verb_names.len(), 2);
        assert!(verb_names.contains(&"store"));
        assert!(verb_names.contains(&"search"));
    }

    #[test]
    fn invocation_from_matches_binds_positional_text_args_in_declared_order() {
        let d = descriptor(
            "core:memory.store",
            "memory",
            vec![
                Binder {
                    name: "key",
                    kind: BinderKind::Text,
                    optional: false,
                },
                Binder {
                    name: "value",
                    kind: BinderKind::Text,
                    optional: false,
                },
            ],
        );
        let invocables = vec![&d];
        let (groups, _) = build_semantic_tree(&invocables);
        let tree = Command::new("cronus").subcommand(groups.into_iter().next().unwrap());

        let matches = tree
            .try_get_matches_from(["cronus", "memory", "store", "fact", "the sky is blue"])
            .expect("parse must succeed");
        let (_, group_matches) = matches.subcommand().unwrap();

        let (resolved, args) =
            invocation_from_matches(group_matches, &invocables).expect("store must resolve");
        assert_eq!(resolved.id.as_str(), "core:memory.store");
        assert_eq!(args.get("key"), Some(&ArgValue::Text("fact".to_string())));
        assert_eq!(
            args.get("value"),
            Some(&ArgValue::Text("the sky is blue".to_string()))
        );
    }

    #[test]
    fn a_flag_binder_is_bound_as_a_named_flag_never_a_positional_word() {
        let d = descriptor(
            "core:role.list",
            "role",
            vec![Binder {
                name: "presets",
                kind: BinderKind::Flag,
                optional: true,
            }],
        );
        let invocables = vec![&d];
        let (groups, _) = build_semantic_tree(&invocables);
        let tree = Command::new("cronus").subcommand(groups.into_iter().next().unwrap());

        // The flag's own name typed as a bare word must NOT satisfy it —
        // if it did, `arg_for` had silently regressed to positional binding.
        let bare_word = tree
            .clone()
            .try_get_matches_from(["cronus", "role", "list", "presets"]);
        assert!(
            bare_word.is_err(),
            "a Flag binder must not accept its own name as a positional argument"
        );

        let matches = tree
            .try_get_matches_from(["cronus", "role", "list", "--presets"])
            .expect("--presets must parse");
        let (_, group_matches) = matches.subcommand().unwrap();
        let (_, args) =
            invocation_from_matches(group_matches, &invocables).expect("list must resolve");
        assert_eq!(args.get("presets"), Some(&ArgValue::Flag));
    }

    #[test]
    fn semantic_shipped_excludes_client_local_host_only_installation_and_retired() {
        let mut registry = InvocableRegistry::new();
        let semantic = descriptor("core:memory.store", "memory", Vec::new());
        let client_local = Invocable {
            locus: Locus::ClientLocal,
            ..descriptor("core:pane.focus", "pane", Vec::new())
        };
        let installation = Invocable {
            locus: Locus::Installation,
            ..descriptor("core:init", "init", Vec::new())
        };
        let retired = Invocable {
            stability: Stability::Retired {
                superseded_by: InvocableId::new("core:memory.store").expect("well-formed test id"),
            },
            ..descriptor("core:memory.old-store", "memory", Vec::new())
        };

        registry
            .register(
                &cronus_core::invocable::Registrant::core(),
                semantic.clone(),
            )
            .unwrap();
        registry
            .register(&cronus_core::invocable::Registrant::core(), client_local)
            .unwrap();
        registry
            .register(&cronus_core::invocable::Registrant::core(), installation)
            .unwrap();
        registry
            .register(&cronus_core::invocable::Registrant::core(), retired)
            .unwrap();

        let shipped = semantic_shipped(&registry);
        assert_eq!(shipped.len(), 1);
        assert_eq!(shipped[0].id, semantic.id);
    }
}
