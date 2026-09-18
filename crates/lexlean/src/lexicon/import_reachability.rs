//! Native diagnostic provenance over resolved package identities and edges.

use std::collections::{BTreeMap, BTreeSet};

use super::package::{LexiconPackage, PackageRef};
use crate::diagnostic::{Diagnostic, DiagnosticDetail};

/// Exact-version reverse reachability. A package-local cycle may fail before
/// the closure exists; only successfully loaded, uniquely identified nodes
/// may extend its independently established member identities.
pub(crate) fn importers(packages: &[LexiconPackage], members: &[PackageRef]) -> Vec<String> {
    let mut counts = BTreeMap::<&str, usize>::new();
    for package in packages {
        *counts.entry(&package.id).or_default() += 1;
    }
    let mut available: BTreeMap<&str, &str> = packages
        .iter()
        .filter(|package| counts[package.id.as_str()] == 1)
        .map(|package| (package.id.as_str(), package.version.as_str()))
        .collect();
    for member in members {
        if !counts.contains_key(member.package.as_str()) {
            available.insert(&member.package, &member.version);
        }
    }
    let mut reverse = BTreeMap::<String, BTreeSet<String>>::new();
    for package in packages {
        if counts[package.id.as_str()] != 1 {
            continue;
        }
        for import in &package.imports {
            if available.get(import.package.as_str()).copied() == Some(import.version.as_str()) {
                reverse
                    .entry(import.to_string())
                    .or_default()
                    .insert(format!("{}@{}", package.id, package.version));
            }
        }
    }
    let mut reached: BTreeSet<String> = members.iter().map(ToString::to_string).collect();
    let mut pending: Vec<String> = reached.iter().cloned().collect();
    while let Some(target) = pending.pop() {
        for parent in reverse.get(&target).into_iter().flatten() {
            if reached.insert(parent.clone()) {
                pending.push(parent.clone());
            }
        }
    }
    reached.into_iter().collect()
}

/// Preserve early package-loading failures while adding only actual resolved
/// reverse edges; no manifest reparsing, failed-package imports, or prose.
pub(crate) fn enrich(packages: &[LexiconPackage], diagnostics: &mut [Diagnostic]) {
    for diagnostic in diagnostics {
        let Some(DiagnosticDetail::PackageImportCycle {
            packages: cycle,
            importers: members,
        }) = diagnostic.detail()
        else {
            continue;
        };
        let Ok(members) = members
            .iter()
            .map(|member| PackageRef::parse(member))
            .collect::<Result<Vec<_>, _>>()
        else {
            continue;
        };
        let detail = DiagnosticDetail::PackageImportCycle {
            packages: cycle.clone(),
            importers: importers(packages, &members),
        };
        *diagnostic = diagnostic.clone().with_detail(detail);
    }
}
