//! Project-wiki regeneration pipeline core (l2-project-wiki §4.2, PW-2/PW-3).
//!
//! Office-owned, curator-driven: on a significant office change, map the
//! change to the affected page kinds, generate each page from ground truth,
//! and write the whole set transactionally through the `WikiCache` seam. The
//! client has no path in here (PW-2). Domain-logic-first: the ground-truth
//! gather + client-language generation is a seam ([`PageRegenerator`]) — the
//! real, model-backed impl lands later; this module owns the orchestration and
//! the all-or-nothing write guarantee.

use std::sync::atomic::{AtomicU64, Ordering};

use cronus_contract::{WikiCache, WikiChangelogEntry, WikiPage, WikiPageKind, now_secs};

/// A significant office change that triggers incremental regeneration
/// (l2-project-wiki §4.2). Never per-agent-turn — only office events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfficeChange {
    BoardItemDone,
    DecisionRecorded,
    DeliverableProduced,
    MilestoneReached,
}

/// Map a change to the page kinds it affects — the "only affected pages"
/// discipline (never a full rebuild on every event). Every change touches the
/// `Overview` and `Changelog`; the rest is change-specific.
pub fn map_event_to_kinds(change: OfficeChange) -> Vec<WikiPageKind> {
    match change {
        OfficeChange::BoardItemDone => {
            vec![
                WikiPageKind::Overview,
                WikiPageKind::Area,
                WikiPageKind::Changelog,
            ]
        }
        OfficeChange::DecisionRecorded => vec![
            WikiPageKind::Overview,
            WikiPageKind::Decisions,
            WikiPageKind::Changelog,
        ],
        OfficeChange::DeliverableProduced => {
            vec![
                WikiPageKind::Overview,
                WikiPageKind::Area,
                WikiPageKind::Changelog,
            ]
        }
        OfficeChange::MilestoneReached => vec![WikiPageKind::Overview, WikiPageKind::Changelog],
    }
}

/// A page produced by the regeneration seam, plus the human-readable summary
/// of what changed (for the changelog).
pub struct RegeneratedPage {
    pub page: WikiPage,
    pub change_summary: String,
}

/// The ground-truth-gather + client-language-generation seam (PW-1/PW-3/PW-4).
///
/// A real implementation reads the board / graph decisions / operational
/// ledger / deliverables and generates cited, plain-language content. `Ok(None)`
/// means there is nothing to regenerate for this kind right now (e.g. no
/// decisions yet) — the page is skipped, not an error. `Err` aborts the whole
/// regeneration **before any write**, so a generation failure never leaves a
/// half-written projection.
pub trait PageRegenerator {
    fn regenerate_page(
        &self,
        office_id: &str,
        kind: WikiPageKind,
    ) -> Result<Option<RegeneratedPage>, String>;
}

/// What a regeneration wrote.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegenReport {
    /// Ids of the pages regenerated (in affected-kind order).
    pub regenerated: Vec<String>,
}

/// Process-unique changelog-id suffix (the contract `MemoryId` id-generation
/// idiom, not per-instance state): guarantees a batch never collides its own
/// changelog primary keys even for two regenerations within the same second.
static CHANGELOG_SEQ: AtomicU64 = AtomicU64::new(0);

/// Regenerate the pages affected by `change` for one office, writing the whole
/// set transactionally (PW-3). A generation failure aborts before any write;
/// the store write is all-or-nothing (`WikiCache::apply_regeneration`). Pages a
/// regenerator reports as `None` are skipped.
pub fn regenerate(
    change: OfficeChange,
    office_id: &str,
    regenerator: &dyn PageRegenerator,
    cache: &dyn WikiCache,
) -> Result<RegenReport, String> {
    let at = now_secs();
    let mut pages: Vec<WikiPage> = Vec::new();
    let mut changelog: Vec<WikiChangelogEntry> = Vec::new();

    for kind in map_event_to_kinds(change) {
        // `?` here means a generation failure returns before any cache write —
        // the prior rows stay intact (transactional at the pipeline level too).
        if let Some(regen) = regenerator.regenerate_page(office_id, kind)? {
            let seq = CHANGELOG_SEQ.fetch_add(1, Ordering::Relaxed);
            changelog.push(WikiChangelogEntry {
                id: format!("cl:{at}:{seq}"),
                office_id: office_id.to_string(),
                page_id: Some(regen.page.id.clone()),
                change: regen.change_summary,
                at,
            });
            pages.push(regen.page);
        }
    }

    if !pages.is_empty() {
        cache.apply_regeneration(&pages, &changelog)?;
    }

    Ok(RegenReport {
        regenerated: pages.into_iter().map(|p| p.id).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    /// A cache that records every applied batch (and never fails), so a test
    /// can assert exactly which pages were written.
    #[derive(Default)]
    struct RecordingCache {
        applied: RefCell<Vec<Vec<String>>>,
    }

    impl WikiCache for RecordingCache {
        fn get_page(&self, _id: &str) -> Result<Option<WikiPage>, String> {
            Ok(None)
        }
        fn apply_regeneration(
            &self,
            pages: &[WikiPage],
            _changelog: &[WikiChangelogEntry],
        ) -> Result<(), String> {
            self.applied
                .borrow_mut()
                .push(pages.iter().map(|p| p.id.clone()).collect());
            Ok(())
        }
    }

    /// Generates one page per requested kind (id = the kind's name), unless a
    /// kind is in `skip` (→ `None`) or `fail_on` (→ `Err`).
    struct MockRegenerator {
        skip: Vec<WikiPageKind>,
        fail_on: Option<WikiPageKind>,
    }

    impl PageRegenerator for MockRegenerator {
        fn regenerate_page(
            &self,
            office_id: &str,
            kind: WikiPageKind,
        ) -> Result<Option<RegeneratedPage>, String> {
            if self.fail_on == Some(kind) {
                return Err(format!("generation failed for {}", kind.as_str()));
            }
            if self.skip.contains(&kind) {
                return Ok(None);
            }
            let page = WikiPage::new(
                kind.as_str(),
                office_id,
                kind,
                kind.as_str(),
                format!("content for {}", kind.as_str()),
            );
            Ok(Some(RegeneratedPage {
                page,
                change_summary: format!("regenerated {}", kind.as_str()),
            }))
        }
    }

    #[test]
    fn event_maps_to_only_the_affected_kinds() {
        assert_eq!(
            map_event_to_kinds(OfficeChange::DecisionRecorded),
            vec![
                WikiPageKind::Overview,
                WikiPageKind::Decisions,
                WikiPageKind::Changelog
            ]
        );
        // A decision does NOT touch the glossary or how-to pages.
        let kinds = map_event_to_kinds(OfficeChange::DecisionRecorded);
        assert!(!kinds.contains(&WikiPageKind::Glossary));
        assert!(!kinds.contains(&WikiPageKind::Howto));
    }

    #[test]
    fn regenerates_only_the_affected_pages_transactionally() {
        let cache = RecordingCache::default();
        let regen = MockRegenerator {
            skip: vec![],
            fail_on: None,
        };

        let report = regenerate(OfficeChange::DecisionRecorded, "office-1", &regen, &cache)
            .expect("regeneration succeeds");

        // Exactly the affected kinds, written in one batch.
        assert_eq!(
            report.regenerated,
            vec!["overview", "decisions", "changelog"]
        );
        let applied = cache.applied.borrow();
        assert_eq!(applied.len(), 1, "one transactional batch");
        assert_eq!(applied[0], vec!["overview", "decisions", "changelog"]);
    }

    #[test]
    fn a_skipped_kind_is_omitted_not_an_error() {
        let cache = RecordingCache::default();
        let regen = MockRegenerator {
            skip: vec![WikiPageKind::Decisions],
            fail_on: None,
        };

        let report = regenerate(OfficeChange::DecisionRecorded, "office-1", &regen, &cache)
            .expect("regeneration succeeds");

        assert_eq!(report.regenerated, vec!["overview", "changelog"]);
        assert_eq!(cache.applied.borrow()[0], vec!["overview", "changelog"]);
    }

    #[test]
    fn a_generation_failure_aborts_before_any_write() {
        let cache = RecordingCache::default();
        let regen = MockRegenerator {
            skip: vec![],
            fail_on: Some(WikiPageKind::Decisions),
        };

        let result = regenerate(OfficeChange::DecisionRecorded, "office-1", &regen, &cache);

        assert!(result.is_err(), "a generation failure surfaces");
        assert!(
            cache.applied.borrow().is_empty(),
            "nothing was written — the prior rows stay intact"
        );
    }
}
