mod cli;
mod commands;
mod generated;
mod installation;
mod output;

use clap::{Command, CommandFactory, FromArgMatches};
use cronus_contract::{Dispatched, Invocable, Outcome, OutcomeValue};
use cronus_core::invocable::Registrant;
use output::OutputFormat;

fn main() -> std::process::ExitCode {
    let installation_invocables = installation::declared_invocables();
    let installation_refs: Vec<&Invocable> = installation_invocables.iter().collect();

    let args: Vec<String> = std::env::args().collect();
    // A bare invocation or a request addressed to the whole surface
    // (`--help`/`-h`/`help`, `--version`/`-V`) is answered from the full
    // composition — this frontend's own installation grammar only owns its
    // *own* help (LH-3), not the top-level listing. Everything else is
    // tried against the installation half first, with no composition at all
    // (LH-1/LH-5): an installation verb must stay answerable even when the
    // composition it would configure is exactly what failed to come up.
    let wants_full_surface = args.len() < 2
        || matches!(
            args[1].as_str(),
            "--help" | "-h" | "help" | "--version" | "-V"
        );

    if !wants_full_surface {
        let (installation_groups, installation_group_names) =
            installation::build_installation_tree(&installation_refs);
        let pre = pre_composition_command(installation_groups);

        match pre.try_get_matches_from(&args) {
            Ok(pre_matches) => {
                if let Some((name, sub_matches)) = pre_matches.subcommand()
                    && installation_group_names.contains(name)
                {
                    let format = pre_matches
                        .get_one::<OutputFormat>("format")
                        .copied()
                        .unwrap_or(OutputFormat::Text);
                    let ctx = output::Context::new(format);
                    let (verb, verb_matches) =
                        sub_matches.subcommand().unwrap_or(("", sub_matches));
                    return exit_code(installation::dispatch(name, verb, verb_matches, &ctx));
                }
                // Matched nothing this half owns (an external subcommand,
                // under `allow_external_subcommands` below) — fall through
                // to the full composition to resolve it.
            }
            Err(e) => {
                // `allow_external_subcommands` means an error here can only
                // come from a *recognized* installation verb's own args (or
                // its own `--help`) — never from an unrecognized top-level
                // name, which is swallowed as an external subcommand
                // instead. This is therefore always a genuine usage failure
                // scoped to this half's own grammar (LH-7): nothing below
                // this line has run, no session opened, nothing journaled.
                e.exit();
            }
        }
    }

    // Compose: the shared registry and dispatcher every surface projects,
    // plus this frontend's own installation descriptors registered into the
    // same catalog (for cross-surface honesty, SP-11 — this is the *other*
    // consumer of the one declaration above). Structured as a real `Result`
    // rather than an early `expect()`: composition cannot actually fail
    // today (no I/O, no extension loading yet), but the distinction LH-7
    // asks for must be representable now rather than retrofitted the day it
    // can.
    let (registry, dispatcher) = match compose(&installation_refs) {
        Ok(pair) => pair,
        Err(reason) => {
            eprintln!("error: composition failed: {reason}");
            return std::process::ExitCode::from(3);
        }
    };

    let semantic = generated::semantic_shipped(&registry);
    let (semantic_groups, generated_group_names) = generated::build_semantic_tree(&semantic);
    let (installation_groups, installation_group_names) =
        installation::build_installation_tree(&installation_refs);

    // The installation half's hand-declared grammar, augmented with
    // whatever the registry currently ships on the semantic half. Built
    // fresh every run — cheap, and the only way a plugin's newly
    // registered verb becomes visible in `--help` without a rebuild.
    let mut command = cli::Cli::command();
    for group in installation_groups {
        command = command.subcommand(group);
    }
    for group in semantic_groups {
        command = command.subcommand(group);
    }
    let matches = command.get_matches();

    let format = matches
        .get_one::<OutputFormat>("format")
        .copied()
        .unwrap_or(OutputFormat::Text);
    let ctx = output::Context::new(format);

    if let Some((name, sub_matches)) = matches.subcommand()
        && installation_group_names.contains(name)
    {
        let (verb, verb_matches) = sub_matches.subcommand().unwrap_or(("", sub_matches));
        return exit_code(installation::dispatch(name, verb, verb_matches, &ctx));
    }

    if let Some((_, group_matches)) =
        generated::matched_generated_group(&matches, &generated_group_names)
        && let Some((invocable, args)) =
            generated::invocation_from_matches(group_matches, &semantic)
    {
        let invocation = cronus_contract::Invocation {
            id: invocable.id.clone(),
            args,
            caller: cronus_contract::Surface::Cli,
        };
        let code = render(dispatcher.dispatch(&registry, &invocation), &ctx);
        return exit_code(code);
    }

    // Not a registry-generated group and not an installation verb — the
    // remaining hand-declared enum. This launcher never resolves a name an
    // extension could define (LH-1/LH-5), so the semantic half is never
    // reached by this fallback.
    match cli::Cli::from_arg_matches(&matches) {
        Ok(args) => exit_code(commands::dispatch(args.command, &ctx)),
        Err(e) => e.exit(),
    }
}

/// The installation half's own pre-composition parser: just enough grammar
/// to recognize and fully parse its own verbs, with everything it does not
/// own passed through untouched rather than refused — ownership of an
/// unrecognized name is not this half's question to answer (LH-5).
fn pre_composition_command(installation_groups: Vec<Command>) -> Command {
    let mut pre = Command::new("cronus")
        .about("Cronus — workflow automation toolkit")
        .disable_version_flag(true)
        .subcommand_required(false)
        .arg_required_else_help(false)
        .allow_external_subcommands(true)
        .arg(installation::format_arg());
    for group in installation_groups {
        pre = pre.subcommand(group);
    }
    pre
}

/// Bring up the registry and dispatcher, then register this frontend's own
/// installation descriptors into the same catalog through the public
/// registration door — no privileged path, the same one a contribution
/// uses. A real `Result`: a registration refusal (a malformed literal
/// descriptor, a duplicate id) is a genuine composition failure, not a
/// panic.
fn compose(
    installation_invocables: &[&Invocable],
) -> Result<
    (
        cronus_core::invocable::InvocableRegistry,
        cronus_core::invocable::Dispatcher,
    ),
    String,
> {
    let (mut registry, dispatcher) =
        cronus_core::invocable_bootstrap::bootstrap(cronus_core::Engine::new());
    for invocable in installation_invocables {
        registry
            .register(&Registrant::core(), (*invocable).clone())
            .map_err(|e| format!("{e:?}"))?;
    }
    Ok((registry, dispatcher))
}

/// Render a dispatched semantic invocation and return its process exit
/// code. Bespoke per invocable for now — the migrated verbs' own handlers
/// return structured `Outcome`s, but no shared, format-uniform renderer
/// exists yet (that is a separate, later task); this reproduces each
/// migrated verb's exact prior text/JSON shape until that renderer replaces
/// it. `Dispatched::Unknown` cannot occur here: this function is only ever
/// called after `invocation_from_matches` already resolved a real
/// descriptor from the same registry `dispatch` immediately consults.
fn render(dispatched: Dispatched, ctx: &output::Context) -> i32 {
    let outcome = match dispatched {
        Dispatched::Ran(outcome) => outcome,
        Dispatched::Unknown => {
            eprintln!("error: internal: dispatched an invocable the registry does not know");
            return 1;
        }
    };
    match outcome {
        Outcome::Rejected(rejection) => {
            eprintln!(
                "error: {} ({:?}): {}",
                rejection.binder, rejection.mode, rejection.detail
            );
            2
        }
        Outcome::Unavailable { reason } => {
            eprintln!("error: {reason}");
            1
        }
        Outcome::Stream(_) => {
            eprintln!("error: internal: a stream outcome has no renderer on this surface yet");
            1
        }
        Outcome::Value(value) => render_value(value, ctx),
    }
}

fn render_value(value: OutcomeValue, ctx: &output::Context) -> i32 {
    match value {
        // `core:memory.store` / `core:memory.forget`: a one-field Record
        // naming the affected entry's id.
        OutcomeValue::Record(fields) if fields.len() == 1 && fields[0].0 == "id" => {
            let id = match &fields[0].1 {
                OutcomeValue::Text(id) => id.as_str(),
                _ => "",
            };
            if ctx.is_json() {
                println!("{{\"result\":\"ok\",\"id\":\"{}\"}}", json_escape(id));
            } else {
                println!("Ok: {id}");
            }
            0
        }
        // `core:memory.search`: a list of `{id, title}` records.
        OutcomeValue::List(items) => {
            if ctx.is_json() {
                let rendered: Vec<String> = items
                    .iter()
                    .map(|item| match item {
                        OutcomeValue::Record(fields) => {
                            let get = |key: &str| {
                                fields
                                    .iter()
                                    .find(|(name, _)| name == key)
                                    .and_then(|(_, v)| match v {
                                        OutcomeValue::Text(s) => Some(s.as_str()),
                                        _ => None,
                                    })
                                    .unwrap_or("")
                            };
                            format!(
                                "{{\"id\":\"{}\",\"title\":\"{}\"}}",
                                json_escape(get("id")),
                                json_escape(get("title"))
                            )
                        }
                        _ => "null".to_string(),
                    })
                    .collect();
                println!("[{}]", rendered.join(","));
            } else if items.is_empty() {
                println!("No results.");
            } else {
                for item in &items {
                    if let OutcomeValue::Record(fields) = item {
                        let get = |key: &str| {
                            fields
                                .iter()
                                .find(|(name, _)| name == key)
                                .and_then(|(_, v)| match v {
                                    OutcomeValue::Text(s) => Some(s.as_str()),
                                    _ => None,
                                })
                                .unwrap_or("")
                        };
                        println!("{}: {}", get("id"), get("title"));
                    }
                }
            }
            0
        }
        OutcomeValue::Text(text) => {
            if ctx.is_json() {
                println!("\"{}\"", json_escape(&text));
            } else {
                println!("{text}");
            }
            0
        }
        OutcomeValue::Empty => 0,
        other => {
            eprintln!("error: internal: no renderer for this outcome shape yet: {other:?}");
            1
        }
    }
}

fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn exit_code(code: i32) -> std::process::ExitCode {
    if code == 0 {
        std::process::ExitCode::SUCCESS
    } else {
        std::process::ExitCode::FAILURE
    }
}
