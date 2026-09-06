//! The installation half of the command surface (§4.1.1): workspace
//! initialization, diagnostics, and developer-office admission — verbs that
//! configure or inspect the product itself rather than act on the user's
//! work.
//!
//! Unlike the semantic half ([`crate::generated`]), this grammar is not
//! generated from the registry — it is declared **once**, right here, and
//! that one declaration has two consumers: [`build_installation_tree`]
//! builds the parser from it before composition even runs, and
//! [`declared_invocables`] feeds the same list into the catalog at
//! composition, so another surface can see these verbs exist and declare
//! that it deliberately does not offer them, instead of looking merely
//! unfinished. A verb declared in a hand-written parser and again as a
//! hand-written descriptor is two statements of one fact — the exact fork
//! this whole command-surface redesign exists to close, one level up from
//! where the semantic half closes it.
//!
//! This is also why an installation verb answers directly, through the
//! functions already proven in [`crate::commands`], rather than through the
//! shared [`cronus_core::invocable::Dispatcher`]: that shared pipeline
//! exists so one handler can serve every surface, and no other surface ever
//! projects the `Installation` locus (only the command line does), so the
//! indirection would buy nothing here.
//!
//! Only a first slice of the eleven installation groups lives here today —
//! `init`, `status`, `doctor`, `restore`, and `dev` — proving the mechanism
//! against both shapes it must support: a **flat** group (its one verb's
//! id-tail equals its group name, so it renders as a single top-level
//! command with no nested verb) and a **nested** group (`dev`, three
//! zero-argument verbs). The remaining groups are follow-up work, not
//! silently dropped.

use std::collections::HashSet;

use clap::{Arg, ArgMatches, Command};
use cronus_contract::{Binder, BinderKind, Invocable, InvocableId, Locus, Stability};

use crate::generated::{arg_for, verb_of};
use crate::output::Context;

fn id(tail: &str) -> InvocableId {
    InvocableId::new(format!("core:{tail}"))
        .expect("literal installation-verb identity must be well-formed — a bug if it isn't")
}

fn text(name: &'static str, optional: bool) -> Binder {
    Binder {
        name,
        kind: BinderKind::Text,
        optional,
    }
}

fn flag(name: &'static str) -> Binder {
    Binder {
        name,
        kind: BinderKind::Flag,
        optional: true,
    }
}

/// The single declaration both the pre-composition parser and the
/// post-composition catalog registration read from.
pub fn declared_invocables() -> Vec<Invocable> {
    vec![
        Invocable {
            id: id("init"),
            name: "Init",
            summary: "Initialize a Cronus workspace in the target directory",
            group: "init",
            locus: Locus::Installation,
            binders: vec![text("path", true)],
            stability: Stability::Shipped,
            journal_raw_input: true,
        },
        Invocable {
            id: id("status"),
            name: "Status",
            summary: "Show the current workspace status",
            group: "status",
            locus: Locus::Installation,
            binders: Vec::new(),
            stability: Stability::Shipped,
            journal_raw_input: true,
        },
        Invocable {
            id: id("doctor"),
            name: "Doctor",
            summary: "Self-healing: run health checks, optionally applying safe repairs",
            group: "doctor",
            locus: Locus::Installation,
            binders: vec![flag("fix")],
            stability: Stability::Shipped,
            journal_raw_input: true,
        },
        Invocable {
            id: id("restore"),
            name: "Restore",
            summary: "Restore a backup into the current state tier",
            group: "restore",
            locus: Locus::Installation,
            binders: vec![text("backup", false)],
            stability: Stability::Shipped,
            journal_raw_input: true,
        },
        Invocable {
            id: id("dev.status"),
            name: "Dev Status",
            summary: "Print the resolved developer-office admission tier for the current directory",
            group: "dev",
            locus: Locus::Installation,
            binders: Vec::new(),
            stability: Stability::Shipped,
            journal_raw_input: true,
        },
        Invocable {
            id: id("dev.admit"),
            name: "Dev Admit",
            summary: "Grant developer-office admission — a human-operator act; run this \
                       yourself, never through an agent-invoked path (DVO-3)",
            group: "dev",
            locus: Locus::Installation,
            binders: Vec::new(),
            stability: Stability::Shipped,
            journal_raw_input: true,
        },
        Invocable {
            id: id("dev.revoke"),
            name: "Dev Revoke",
            summary: "Revoke developer-office admission",
            group: "dev",
            locus: Locus::Installation,
            binders: Vec::new(),
            stability: Stability::Shipped,
            journal_raw_input: true,
        },
    ]
}

/// A nested group's own about text — declared here rather than invented as
/// a generic `"<group> operations"` fallback, since a nested installation
/// group deserves the same care a flat one gets from its one verb's own
/// summary. Groups not yet listed here (the six not migrated this pass)
/// fall back to the generic text until they join this list.
fn group_about(group: &str) -> Option<&'static str> {
    match group {
        "dev" => {
            Some("Self-hosting developer office: conditional, human-admitted, canonical-repo-only")
        }
        _ => None,
    }
}

/// Build the clap subcommand tree for the installation half. Unlike the
/// semantic tree (always nested, one subcommand level per group), a group
/// with exactly one verb whose id-tail equals its own group name — no
/// `"<group>."` prefix to strip — renders **flat**: the group's own
/// top-level `Command` carries that verb's args directly, matching the
/// existing `cronus doctor --fix` / `cronus restore <backup>` shape rather
/// than inventing `cronus doctor doctor`.
pub fn build_installation_tree(invocables: &[&Invocable]) -> (Vec<Command>, HashSet<String>) {
    let mut by_group: std::collections::HashMap<&str, Vec<&Invocable>> =
        std::collections::HashMap::new();
    for invocable in invocables {
        by_group.entry(invocable.group).or_default().push(invocable);
    }

    let mut group_order: Vec<&str> = by_group.keys().copied().collect();
    group_order.sort_unstable();

    let mut group_names = HashSet::new();
    let mut groups = Vec::new();

    for group in group_order {
        let members = &by_group[group];
        group_names.insert(group.to_string());

        if members.len() == 1 && verb_of(members[0]) == group {
            let mut flat = Command::new(group.to_string()).about(members[0].summary);
            for binder in &members[0].binders {
                flat = flat.arg(arg_for(binder));
            }
            groups.push(flat);
            continue;
        }

        let about = group_about(group)
            .map(str::to_string)
            .unwrap_or_else(|| format!("{group} operations"));
        let mut nested = Command::new(group.to_string()).about(about);
        for invocable in members {
            let mut verb = Command::new(verb_of(invocable).to_string()).about(invocable.summary);
            for binder in &invocable.binders {
                verb = verb.arg(arg_for(binder));
            }
            nested = nested.subcommand(verb);
        }
        groups.push(nested);
    }

    (groups, group_names)
}

/// The shared `--format`/`-f` global, duplicated here rather than shared
/// with `Cli`'s derive-generated definition: the two are genuinely two
/// consumers of the same **flag**, not the verb grammar this module's own
/// "declared once" property is about, and it is the one launcher-owned flag
/// LH-1 names explicitly ("which configuration overlays to apply").
pub fn format_arg() -> Arg {
    Arg::new("format")
        .long("format")
        .short('f')
        .global(true)
        .value_parser(clap::value_parser!(crate::output::OutputFormat))
        .default_value("text")
}

/// Dispatch a resolved installation verb directly to its already-proven
/// handler in [`crate::commands`] — no shared `Dispatcher`/`Outcome`
/// indirection, for the reason the module doc gives. Returns the same exit
/// code the handler itself already returned before this migration.
pub fn dispatch(group: &str, verb: &str, matches: &ArgMatches, ctx: &Context) -> i32 {
    match (group, verb) {
        ("init", _) => {
            let path = matches
                .get_one::<String>("path")
                .map(std::path::PathBuf::from);
            crate::commands::init::run(path, ctx)
        }
        ("status", _) => crate::commands::status::run(ctx),
        ("doctor", _) => {
            let fix = matches.get_flag("fix");
            crate::commands::doctor::run(fix, ctx)
        }
        ("restore", _) => {
            let backup = matches
                .get_one::<String>("backup")
                .map(String::as_str)
                .unwrap_or_default();
            crate::commands::backup_cmd::restore(backup, ctx)
        }
        ("dev", "status") => crate::commands::dev_office_cmd::status(ctx),
        ("dev", "admit") => crate::commands::dev_office_cmd::admit(ctx),
        ("dev", "revoke") => crate::commands::dev_office_cmd::revoke(ctx),
        _ => {
            eprintln!(
                "error: internal: no installation handler wired for {group} {verb} — a bug in this module's own dispatch table, not a caller condition"
            );
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet as Set;

    use cronus_core::invocable::{InvocableRegistry, Registrant};

    use super::*;

    #[test]
    fn a_flat_group_renders_as_one_top_level_command_with_no_nested_verb() {
        let invocables = declared_invocables();
        let refs: Vec<&Invocable> = invocables.iter().filter(|i| i.group == "doctor").collect();
        let (groups, names) = build_installation_tree(&refs);
        assert_eq!(names, Set::from(["doctor".to_string()]));
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].get_name(), "doctor");
        assert_eq!(
            groups[0].get_subcommands().count(),
            0,
            "a flat group must carry its one verb's args directly, not nest a same-named subcommand"
        );
        assert!(groups[0].get_arguments().any(|a| a.get_id() == "fix"));
    }

    #[test]
    fn a_multi_verb_group_renders_nested_exactly_like_the_semantic_half() {
        let invocables = declared_invocables();
        let refs: Vec<&Invocable> = invocables.iter().filter(|i| i.group == "dev").collect();
        let (groups, _) = build_installation_tree(&refs);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].get_name(), "dev");
        let verbs: Vec<&str> = groups[0].get_subcommands().map(|c| c.get_name()).collect();
        assert_eq!(verbs.len(), 3);
        for expected in ["status", "admit", "revoke"] {
            assert!(verbs.contains(&expected), "missing dev verb: {expected}");
        }
    }

    #[test]
    fn a_required_text_binder_rejects_the_verb_with_no_value() {
        let invocables = declared_invocables();
        let refs: Vec<&Invocable> = invocables.iter().filter(|i| i.group == "restore").collect();
        let (groups, _) = build_installation_tree(&refs);
        let tree = Command::new("cronus").subcommand(groups.into_iter().next().unwrap());
        assert!(
            tree.clone()
                .try_get_matches_from(["cronus", "restore"])
                .is_err(),
            "restore's required backup id must not be optional"
        );
        let matches = tree
            .try_get_matches_from(["cronus", "restore", "b-1"])
            .expect("restore b-1 must parse");
        let (_, sub) = matches.subcommand().unwrap();
        assert_eq!(
            sub.get_one::<String>("backup").map(String::as_str),
            Some("b-1")
        );
    }

    /// The literal Verify criterion this task names: the pre-composition
    /// parser and the post-composition catalog registration are two
    /// **consumers** of the same declaration, not two declarations — proven
    /// by comparing the verb set each one actually produces.
    #[test]
    fn the_parser_and_the_registered_catalog_advertise_exactly_the_same_verbs() {
        let invocables = declared_invocables();
        let invocable_refs: Vec<&Invocable> = invocables.iter().collect();

        let (groups, _) = build_installation_tree(&invocable_refs);
        let mut parser_verbs: Set<(String, String)> = Set::new();
        for group in &groups {
            if group.get_subcommands().count() == 0 {
                parser_verbs.insert((group.get_name().to_string(), group.get_name().to_string()));
            } else {
                for verb in group.get_subcommands() {
                    parser_verbs
                        .insert((group.get_name().to_string(), verb.get_name().to_string()));
                }
            }
        }

        let mut registry = InvocableRegistry::new();
        for invocable in invocables.iter().cloned() {
            registry
                .register(&Registrant::core(), invocable)
                .expect("every declared installation invocable must register cleanly");
        }
        let catalog_verbs: Set<(String, String)> = registry
            .all()
            .filter(|invocable| matches!(invocable.locus, Locus::Installation))
            .map(|invocable| (invocable.group.to_string(), verb_of(invocable).to_string()))
            .collect();

        assert_eq!(
            parser_verbs, catalog_verbs,
            "the launcher parser and the registered catalog must advertise exactly the same \
             verbs — any difference means the two are drifting from separately maintained lists \
             rather than deriving from the one declaration"
        );
    }
}
