use std::sync::Arc;

use cronus_codegraph::extractor::{Extractor, RegexExtractor};
use cronus_codegraph::index::CodeIndex;
use cronus_contract::{Binder, BinderKind, Invocable, Locus, Outcome, OutcomeValue, Stability};
use cronus_domain::invocable::{Dispatcher, InvocableRegistry, Registrant};

use super::{core_id, text_arg};

pub(super) fn register(registry: &mut InvocableRegistry, dispatcher: &mut Dispatcher) {
    register_index(registry, dispatcher);
    register_search(registry, dispatcher);
}

fn register_index(registry: &mut InvocableRegistry, dispatcher: &mut Dispatcher) {
    let id = core_id("codegraph.index");
    let invocable = Invocable {
        id: id.clone(),
        name: "Index",
        summary: "Index a directory or file into the code graph.",
        group: "codegraph",
        locus: Locus::Semantic,
        binders: vec![Binder {
            name: "path",
            kind: BinderKind::Text,
            optional: false,
        }],
        stability: Stability::Shipped,
        journal_raw_input: true,
    };
    registry.register(&Registrant::core(), invocable).expect(
        "core:codegraph.index registers cleanly at bootstrap — a duplicate id here is a bug",
    );
    dispatcher.attach(
        id,
        Arc::new(|args| {
            let path = std::path::PathBuf::from(text_arg(args, "path"));
            // Ephemeral per call — `CodeIndex::open_in_memory` is the same
            // shape `MemoryStore::open_in_memory` already carries, and the
            // same residual applies: a later `search` in a separate
            // invocation cannot see what an earlier `index` stored. Not
            // fixed here — preserved exactly (SP-9/SP-10), recorded
            // separately (`CODEGRAPH_INDEX_EPHEMERAL_PER_CALL`).
            let index = match CodeIndex::open_in_memory() {
                Ok(index) => index,
                Err(e) => {
                    return Outcome::Unavailable {
                        reason: e.to_string(),
                    };
                }
            };
            let extractor = RegexExtractor;
            let mut total = 0usize;
            if path.is_file() {
                total += index_file(&index, &path, &extractor);
            } else if path.is_dir()
                && let Ok(entries) = std::fs::read_dir(&path)
            {
                for entry in entries.filter_map(|e| e.ok()) {
                    let p = entry.path();
                    if p.extension().and_then(|e| e.to_str()) == Some("rs") {
                        total += index_file(&index, &p, &extractor);
                    }
                }
            }
            Outcome::Value(OutcomeValue::Record(vec![
                (
                    "path".to_string(),
                    OutcomeValue::Text(path.display().to_string()),
                ),
                ("symbols".to_string(), OutcomeValue::Integer(total as i64)),
            ]))
        }),
    );
}

fn index_file(index: &CodeIndex, path: &std::path::Path, extractor: &RegexExtractor) -> usize {
    let source = std::fs::read_to_string(path).unwrap_or_default();
    let syms = extractor.extract(&source);
    let n = syms.len();
    let _ = index.index_symbols(&path.display().to_string(), &syms);
    n
}

fn register_search(registry: &mut InvocableRegistry, dispatcher: &mut Dispatcher) {
    let id = core_id("codegraph.search");
    let invocable = Invocable {
        id: id.clone(),
        name: "Search",
        summary: "Search the code graph by keyword.",
        group: "codegraph",
        locus: Locus::Semantic,
        binders: vec![Binder {
            name: "query",
            kind: BinderKind::Text,
            optional: false,
        }],
        stability: Stability::Shipped,
        journal_raw_input: true,
    };
    registry.register(&Registrant::core(), invocable).expect(
        "core:codegraph.search registers cleanly at bootstrap — a duplicate id here is a bug",
    );
    dispatcher.attach(
        id,
        Arc::new(|args| {
            let query = text_arg(args, "query");
            let index = match CodeIndex::open_in_memory() {
                Ok(index) => index,
                Err(e) => {
                    return Outcome::Unavailable {
                        reason: e.to_string(),
                    };
                }
            };
            match index.search(query, 10) {
                Ok(results) => Outcome::Value(OutcomeValue::List(
                    results
                        .into_iter()
                        .map(|s| {
                            OutcomeValue::Record(vec![
                                ("name".to_string(), OutcomeValue::Text(s.name)),
                                ("file".to_string(), OutcomeValue::Text(s.file)),
                                ("line".to_string(), OutcomeValue::Integer(s.line as i64)),
                            ])
                        })
                        .collect(),
                )),
                Err(e) => Outcome::Unavailable {
                    reason: e.to_string(),
                },
            }
        }),
    );
}
