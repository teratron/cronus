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
//! Ten of the eleven installation groups live here — `init`, `status`,
//! `doctor`, `restore`, `dev`, `workspace`, `backup`, `activation`,
//! `archetype`, and `registry` — covering three tree shapes: **flat** (a
//! group's one verb's id-tail equals its group name, so it renders as a
//! single top-level command with no nested verb), **nested** (`dev`,
//! `workspace`, `backup`, `activation`, `archetype`, `registry` — a group
//! command containing verb subcommands), and a **named value flag**
//! (`--actor cli`, `--mode login`), which `BinderKind::NamedText` exists to
//! express — `Binder` had no positional-vs-named-value distinction before
//! this module needed one for `workspace create --name/--path`, `backup
//! create --to`, `activation enable --mode`, and `archetype create --from`.
//! `ext` (with its own nested `skill` sub-group — a real third level of
//! nesting this module's two-level tree builder does not yet handle) is the
//! one remaining group, follow-up work, not silently dropped.

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

/// A value bound as a named `--name <value>` flag rather than positionally.
/// A missing optional one reads back as absent from `ArgValues`, and the
/// handler applies its own default the same way an absent positional `Text`
/// binder's caller already would — `Binder` carries no default-value slot
/// of its own, matching the minimalism the rest of this mechanism already
/// holds to.
fn named_text(name: &'static str, optional: bool) -> Binder {
    Binder {
        name,
        kind: BinderKind::NamedText,
        optional,
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
        Invocable {
            id: id("workspace.create"),
            name: "Workspace Create",
            summary: "Create a new workspace",
            group: "workspace",
            locus: Locus::Installation,
            binders: vec![
                text("id", false),
                named_text("name", true),
                named_text("path", true),
            ],
            stability: Stability::Shipped,
            journal_raw_input: true,
        },
        Invocable {
            id: id("workspace.list"),
            name: "Workspace List",
            summary: "List all workspaces",
            group: "workspace",
            locus: Locus::Installation,
            binders: Vec::new(),
            stability: Stability::Shipped,
            journal_raw_input: true,
        },
        Invocable {
            id: id("workspace.switch"),
            name: "Workspace Switch",
            summary: "Switch the active workspace",
            group: "workspace",
            locus: Locus::Installation,
            binders: vec![text("id", false)],
            stability: Stability::Shipped,
            journal_raw_input: true,
        },
        Invocable {
            id: id("workspace.delete"),
            name: "Workspace Delete",
            summary: "Delete a workspace",
            group: "workspace",
            locus: Locus::Installation,
            binders: vec![text("id", false)],
            stability: Stability::Shipped,
            journal_raw_input: true,
        },
        Invocable {
            id: id("workspace.check"),
            name: "Workspace Check",
            summary: "Check the status of a workspace",
            group: "workspace",
            locus: Locus::Installation,
            binders: vec![text("id", false)],
            stability: Stability::Shipped,
            journal_raw_input: true,
        },
        Invocable {
            id: id("backup.create"),
            name: "Backup Create",
            summary: "Create a backup",
            group: "backup",
            locus: Locus::Installation,
            binders: vec![named_text("to", true), flag("include_logs")],
            stability: Stability::Shipped,
            journal_raw_input: true,
        },
        Invocable {
            id: id("backup.list"),
            name: "Backup List",
            summary: "List backups under the state tier's backups/ directory",
            group: "backup",
            locus: Locus::Installation,
            binders: Vec::new(),
            stability: Stability::Shipped,
            journal_raw_input: true,
        },
        Invocable {
            id: id("activation.status"),
            name: "Activation Status",
            summary: "Print the observed activation state — read from the OS, never a \
                       remembered value (BA-8)",
            group: "activation",
            locus: Locus::Installation,
            binders: Vec::new(),
            stability: Stability::Shipped,
            journal_raw_input: true,
        },
        Invocable {
            id: id("activation.enable"),
            name: "Activation Enable",
            summary: "Register background activation for a mode (BA-5: an autonomy grant, \
                       not a preference — disclosed and confirmed before it takes effect)",
            group: "activation",
            locus: Locus::Installation,
            binders: vec![
                named_text("mode", false),
                flag("acknowledge_unattended_execution"),
            ],
            stability: Stability::Shipped,
            journal_raw_input: true,
        },
        Invocable {
            id: id("activation.disable"),
            name: "Activation Disable",
            summary: "Remove whatever activation registration is currently active (BA-7: \
                       removed and verified, never left partially registered)",
            group: "activation",
            locus: Locus::Installation,
            binders: Vec::new(),
            stability: Stability::Shipped,
            journal_raw_input: true,
        },
        Invocable {
            id: id("archetype.list"),
            name: "Archetype List",
            summary: "List archetypes — the shipped catalog, or the office's active one",
            group: "archetype",
            locus: Locus::Installation,
            binders: vec![flag("catalog"), flag("active")],
            stability: Stability::Shipped,
            journal_raw_input: true,
        },
        Invocable {
            id: id("archetype.info"),
            name: "Archetype Info",
            summary: "Show one archetype's pool, shape, and seed (or its blocked reason)",
            group: "archetype",
            locus: Locus::Installation,
            binders: vec![text("id", false), flag("deviations")],
            stability: Stability::Shipped,
            journal_raw_input: true,
        },
        Invocable {
            id: id("archetype.set"),
            name: "Archetype Set",
            summary: "Apply an archetype, or return to the archetype-free default. Changes \
                       what the manager expects, never staff — non-destructive by construction",
            group: "archetype",
            locus: Locus::Installation,
            binders: vec![text("id", true), flag("clear")],
            stability: Stability::Shipped,
            journal_raw_input: true,
        },
        Invocable {
            id: id("archetype.create"),
            name: "Archetype Create",
            summary: "Create a custom archetype by copying a preset into the state tier",
            group: "archetype",
            locus: Locus::Installation,
            binders: vec![text("name", false), named_text("from", false)],
            stability: Stability::Shipped,
            journal_raw_input: true,
        },
        Invocable {
            id: id("registry.list"),
            name: "Registry List",
            summary: "List all agent definitions",
            group: "registry",
            locus: Locus::Installation,
            binders: Vec::new(),
            stability: Stability::Shipped,
            journal_raw_input: true,
        },
        Invocable {
            id: id("registry.show"),
            name: "Registry Show",
            summary: "Show an agent definition",
            group: "registry",
            locus: Locus::Installation,
            binders: vec![text("name", false)],
            stability: Stability::Shipped,
            journal_raw_input: true,
        },
        Invocable {
            id: id("registry.create"),
            name: "Registry Create",
            summary: "Create a custom agent entry",
            group: "registry",
            locus: Locus::Installation,
            binders: vec![text("name", false), text("description", false)],
            stability: Stability::Shipped,
            journal_raw_input: true,
        },
        Invocable {
            id: id("registry.disable"),
            name: "Registry Disable",
            summary: "Disable an agent",
            group: "registry",
            locus: Locus::Installation,
            binders: vec![text("name", false)],
            stability: Stability::Shipped,
            journal_raw_input: true,
        },
        Invocable {
            id: id("registry.enable"),
            name: "Registry Enable",
            summary: "Enable a previously disabled agent",
            group: "registry",
            locus: Locus::Installation,
            binders: vec![text("name", false)],
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
        "workspace" => Some("Manage named workspaces"),
        "backup" => Some("Back up the state tier (secrets and cache excluded)"),
        "activation" => Some("Manage background activation"),
        "archetype" => Some("Office archetypes: a prior on staffing, never a roster"),
        "registry" => Some("Agent registry: list and manage agent definitions"),
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
        ("workspace", "create") => {
            let id = matches.get_one::<String>("id").cloned().unwrap_or_default();
            let name = matches.get_one::<String>("name").cloned();
            let path = matches
                .get_one::<String>("path")
                .map(std::path::PathBuf::from);
            crate::commands::workspace::create(id, name, path, ctx)
        }
        ("workspace", "list") => crate::commands::workspace::list(ctx),
        ("workspace", "switch") => {
            let id = matches.get_one::<String>("id").cloned().unwrap_or_default();
            crate::commands::workspace::switch(id, ctx)
        }
        ("workspace", "delete") => {
            let id = matches.get_one::<String>("id").cloned().unwrap_or_default();
            crate::commands::workspace::delete(id, ctx)
        }
        ("workspace", "check") => {
            let id = matches.get_one::<String>("id").cloned().unwrap_or_default();
            crate::commands::workspace::check(id, ctx)
        }
        ("backup", "create") => {
            let to = matches
                .get_one::<String>("to")
                .map(std::path::PathBuf::from);
            let include_logs = matches.get_flag("include_logs");
            crate::commands::backup_cmd::create(to, include_logs, ctx)
        }
        ("backup", "list") => crate::commands::backup_cmd::list(ctx),
        ("activation", "status") => crate::commands::activation_cmd::status(ctx),
        ("activation", "enable") => {
            let mode = match matches.get_one::<String>("mode").map(String::as_str) {
                Some("login") => crate::cli::ActivationModeArg::Login,
                Some("system") => crate::cli::ActivationModeArg::System,
                other => {
                    // An application-level refusal, not a clap-level usage
                    // failure: the flag parsed fine as text, dispatch ran,
                    // and the value it ran with was one it does not accept
                    // — the same "ran and refused" shape every other
                    // handler in this module reports with exit 1.
                    eprintln!(
                        "error: --mode must be 'login' or 'system', got {:?}",
                        other.unwrap_or("<none>")
                    );
                    return 1;
                }
            };
            let acknowledged = matches.get_flag("acknowledge_unattended_execution");
            crate::commands::activation_cmd::enable(mode, acknowledged, ctx)
        }
        ("activation", "disable") => crate::commands::activation_cmd::disable(ctx),
        ("archetype", "list") => {
            let catalog = matches.get_flag("catalog");
            let active = matches.get_flag("active");
            crate::commands::archetype_cmd::list(catalog, active)
        }
        ("archetype", "info") => {
            let id = matches.get_one::<String>("id").cloned().unwrap_or_default();
            let deviations = matches.get_flag("deviations");
            crate::commands::archetype_cmd::info(&id, deviations)
        }
        ("archetype", "set") => {
            let id = matches.get_one::<String>("id").cloned();
            let clear = matches.get_flag("clear");
            crate::commands::archetype_cmd::set(id, clear, ctx)
        }
        ("archetype", "create") => {
            let name = matches
                .get_one::<String>("name")
                .cloned()
                .unwrap_or_default();
            let from = matches
                .get_one::<String>("from")
                .cloned()
                .unwrap_or_default();
            crate::commands::archetype_cmd::create(&name, &from)
        }
        ("registry", "list") => crate::commands::registry::list(ctx),
        ("registry", "show") => {
            let name = matches
                .get_one::<String>("name")
                .cloned()
                .unwrap_or_default();
            crate::commands::registry::show(name, ctx)
        }
        ("registry", "create") => {
            let name = matches
                .get_one::<String>("name")
                .cloned()
                .unwrap_or_default();
            let description = matches
                .get_one::<String>("description")
                .cloned()
                .unwrap_or_default();
            crate::commands::registry::create(name, description, ctx)
        }
        ("registry", "disable") => {
            let name = matches
                .get_one::<String>("name")
                .cloned()
                .unwrap_or_default();
            crate::commands::registry::disable(name, ctx)
        }
        ("registry", "enable") => {
            let name = matches
                .get_one::<String>("name")
                .cloned()
                .unwrap_or_default();
            crate::commands::registry::enable(name, ctx)
        }
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

    /// `BinderKind::NamedText` — proves the extension end to end: bound as
    /// `--name <value>`, never positionally, and a required one refuses the
    /// verb entirely when absent, exactly like a required positional would.
    #[test]
    fn a_named_text_binder_is_bound_as_a_named_flag_never_a_positional_word() {
        let invocables = declared_invocables();
        let refs: Vec<&Invocable> = invocables
            .iter()
            .filter(|i| i.group == "archetype")
            .collect();
        let (groups, _) = build_installation_tree(&refs);
        let tree = Command::new("cronus").subcommand(groups.into_iter().next().unwrap());

        // `create` requires `name` (positional) and `--from` (named) — the
        // bare word form must not satisfy `--from`.
        assert!(
            tree.clone()
                .try_get_matches_from(["cronus", "archetype", "create", "custom", "preset-a"])
                .is_err(),
            "a NamedText binder must not accept its value as a second positional word"
        );

        let matches = tree
            .try_get_matches_from([
                "cronus",
                "archetype",
                "create",
                "custom",
                "--from",
                "preset-a",
            ])
            .expect("create custom --from preset-a must parse");
        let (_, group_matches) = matches.subcommand().unwrap();
        let (_, verb_matches) = group_matches.subcommand().unwrap();
        assert_eq!(
            verb_matches.get_one::<String>("from").map(String::as_str),
            Some("preset-a")
        );
    }

    /// An optional `NamedText` binder (`workspace create --name`) must
    /// parse fine when absent — the handler, not the parser, supplies a
    /// default, matching every other optional binder's contract.
    #[test]
    fn an_optional_named_text_binder_is_absent_when_not_supplied() {
        let invocables = declared_invocables();
        let refs: Vec<&Invocable> = invocables
            .iter()
            .filter(|i| i.group == "workspace")
            .collect();
        let (groups, _) = build_installation_tree(&refs);
        let tree = Command::new("cronus").subcommand(groups.into_iter().next().unwrap());

        let matches = tree
            .try_get_matches_from(["cronus", "workspace", "create", "my-ws"])
            .expect("create with no --name/--path must still parse");
        let (_, group_matches) = matches.subcommand().unwrap();
        let (_, verb_matches) = group_matches.subcommand().unwrap();
        assert_eq!(
            verb_matches.get_one::<String>("id").map(String::as_str),
            Some("my-ws")
        );
        assert_eq!(verb_matches.get_one::<String>("name"), None);
        assert_eq!(verb_matches.get_one::<String>("path"), None);
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
