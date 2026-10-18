//! A per-document census of the whole corpus, by level and version.
//!
//! It reads every document of `./syntactic` (or the directory given as the first argument),
//! resolves its object model, validates it and prints one `<count>\t<level>/<version>|<rule>` line
//! per (level, version, rule) plus a `documents=... models=... panics=...` header. It is the
//! instrument behind every per-version number in `notes/g5-report.md` sections 5-7: it sees the
//! levels the harness does not compare, which is what makes a level's own measurement possible
//! before its comparison is landed.
//!
//!     cargo run --release --example probe-c24 > /tmp/census.txt
//!     grep -E "^[0-9]+\t2/" /tmp/census.txt      # the Level 2 classes
use biodivine_lib_sbml::xml::{OptionalXmlChild, XmlProperty};
use biodivine_lib_sbml::Sbml;
use std::collections::BTreeMap;

fn main() {
    let root = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "syntactic".to_string());
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut models = 0usize;
    let mut docs = 0usize;
    let mut panics = 0usize;
    let mut unreadable = 0usize;
    let mut dirs: Vec<_> = std::fs::read_dir(&root)
        .expect("the corpus directory")
        .map(|it| it.unwrap().path())
        .collect();
    dirs.sort();
    for dir in dirs {
        if !dir.is_dir() {
            continue;
        }
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().map(|it| it != "xml").unwrap_or(true) {
                continue;
            }
            docs += 1;
            let Ok(src) = std::fs::read_to_string(&path) else {
                unreadable += 1;
                continue;
            };
            let Ok(doc) = Sbml::read_str(&src) else {
                unreadable += 1;
                continue;
            };
            let key = match (doc.level().get_raw(), doc.version().get_raw()) {
                (Some(level), Some(version)) => format!("{level}/{version}"),
                _ => "?/?".to_string(),
            };
            if doc.model().get().is_some() {
                models += 1;
            }
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| doc.validate())) {
                Ok(Ok(issues)) => {
                    for issue in issues {
                        *counts.entry(format!("{key}|{}", issue.rule())).or_default() += 1;
                    }
                }
                _ => {
                    *counts.entry(format!("{key}|PANIC")).or_default() += 1;
                    panics += 1;
                }
            }
        }
    }
    println!("documents={docs} models={models} panics={panics} unreadable={unreadable}");
    for (key, count) in counts.iter() {
        println!("{count}\t{key}");
    }
}
