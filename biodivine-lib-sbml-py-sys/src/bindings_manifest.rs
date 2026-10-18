//! The manifest test: `BINDINGS.md` against the module this crate registers.
//!
//! `BINDINGS.md` states, for every public item of the library, whether the bindings expose it and
//! under which Python name. A generated file can be *stale* (the `--check` gate of
//! `scripts/public_surface.py` catches that) but it can also be *wrong*: a row can claim a Python
//! name that the wrapper crate does not actually define. This test is what makes the claim
//! checkable, because it builds the very module the aggregate extension builds and looks every
//! bound name up in it.
//!
//! Two properties are asserted:
//!
//!  * every `bound` row whose rule is in [`IMPLEMENTED_RULES`] resolves in the module (a class, a
//!    class member, a module function or a module constant);
//!  * every `not bound` row carries a rule that the manifest's own rule table explains.
//!
//! [`IMPLEMENTED_RULES`] is the list of rule *classes* the wrappers implement. A rule that is not
//! in it is a remainder: the manifest may call its items bound, but this test does not claim they
//! resolve, and the report of the unit names what is left. Adding a rule to the manifest without
//! adding it here fails the test (`a_rule_the_manifest_binds_must_be_implemented`), so the list
//! cannot silently fall behind, and adding it here without generating the wrappers fails
//! `every_bound_name_resolves`.

use std::collections::BTreeSet;

use pyo3::prelude::*;
use pyo3::types::PyModule;

/// The rule classes of `BINDINGS.md` whose bound names this crate implements.
///
/// Every rule the manifest uses with the status `bound` is listed here: the wrappers cover the
/// whole public surface of the library.
const IMPLEMENTED_RULES: &[&str] = &[
    "bound-type-member",
    "child",
    "constructor",
    "document",
    "enum-variant",
    "error-enum",
    "exception",
    "extension-error",
    "foreign",
    "free-fn",
    "lookup",
    "math-accessor",
    "math-node-type",
    "namespace-const",
    "property",
    "root-namespaces",
    "rule-set",
    "sbase",
    "sbml-object-identity",
    "sbml-units",
    "subtype-cast",
    "trait-accessor",
    "validation-issue",
    "value-enum",
    "version-const",
    "wrapper-accessor",
    "wrapper-type",
    "xml-default",
    "xml-wrapper",
];

/// One row of the manifest table.
#[derive(Debug)]
struct Row {
    item: String,
    status: String,
    python: String,
    rule: String,
}

/// Reads the manifest region of `BINDINGS.md` and the rules it names.
fn manifest() -> (Vec<Row>, BTreeSet<String>) {
    let text = std::fs::read_to_string("BINDINGS.md").expect(
        "BINDINGS.md is next to the crate's manifest; run `make bindings` if it is missing",
    );
    let mut rows = Vec::new();
    let mut known_rules = BTreeSet::new();
    let mut in_table = false;
    let mut in_manifest = false;
    for line in text.lines() {
        if line.contains("manifest:begin") {
            in_manifest = true;
            continue;
        }
        if line.contains("manifest:end") {
            in_manifest = false;
            continue;
        }
        if !in_manifest {
            // The rule table: `| `rule-id` | matches | status | python | reason | count |`.
            if let Some(cells) = parse_row(line) {
                // The rule table ends with the number of items the rule classified.
                if cells.len() == 6 && cells[5].parse::<usize>().is_ok() {
                    known_rules.insert(cells[0].clone());
                }
            }
            continue;
        }
        let _ = &mut in_table;
        if let Some(cells) = parse_row(line) {
            if cells.len() == 5 {
                rows.push(Row {
                    item: cells[0].clone(),
                    status: cells[2].clone(),
                    python: cells[3].clone(),
                    rule: cells[4].clone(),
                });
            }
        }
    }
    assert!(
        rows.len() > 1000,
        "the manifest table looks truncated: {} rows",
        rows.len()
    );
    assert!(
        !known_rules.is_empty(),
        "the manifest has no rule table to explain the reasons"
    );
    // The explicit-exception table states its reasons row by row instead of by rule.
    known_rules.insert("exception".to_string());
    (rows, known_rules)
}

/// Splits a markdown table row into its cells, dropping the backticks the table uses for code.
fn parse_row(line: &str) -> Option<Vec<String>> {
    let line = line.trim();
    if !line.starts_with("| `") {
        return None;
    }
    let cells: Vec<String> = line
        .trim_matches('|')
        .split('|')
        .map(|cell| cell.trim().trim_matches('`').to_string())
        .collect();
    Some(cells)
}

/// Checks that one Python name exists in the module it belongs to.
///
/// A name prefixed with `xml_dom.` belongs to the XML DOM bindings, which the SBML module reuses
/// rather than duplicates; the test builds that module too and looks the name up there.
fn resolves(py: Python<'_>, module: &Bound<'_, PyModule>, name: &str) -> bool {
    let name = name.trim().trim_end_matches("()");
    if let Some(rest) = name.strip_prefix("xml_dom.") {
        let xml_dom =
            PyModule::new(py, "biodivine_sys.xml_dom").expect("the module can be created");
        biodivine_lib_xml_dom_sys::register(&xml_dom).expect("register fills the module");
        return xml_dom.hasattr(rest).unwrap_or(false);
    }
    let (head, member) = match name.rsplit_once('.') {
        Some((head, member)) => (head, Some(member)),
        None => (name, None),
    };
    let Ok(target) = module.getattr(head) else {
        return false;
    };
    match member {
        None => true,
        Some(member) => target.hasattr(member).unwrap_or(false),
    }
}

#[test]
fn every_bound_name_resolves() {
    Python::attach(|py| {
        let module = PyModule::new(py, "biodivine_sys.sbml").expect("the module can be created");
        crate::register(&module).expect("register fills the module");
        let (rows, _rules) = manifest();
        let mut missing: Vec<String> = Vec::new();
        let mut checked = 0;
        for row in &rows {
            if row.status != "bound" || !IMPLEMENTED_RULES.contains(&row.rule.as_str()) {
                continue;
            }
            for name in row.python.split(';') {
                let name = name.trim();
                if name.is_empty() || name == "-" {
                    missing.push(format!("{}: empty Python name", row.item));
                    continue;
                }
                checked += 1;
                if !resolves(py, &module, name) {
                    missing.push(format!("{}: `{name}` is not in the module", row.item));
                }
            }
        }
        assert!(checked > 1000, "the check covered only {checked} names");
        assert!(
            missing.is_empty(),
            "{} bound names do not resolve in the module:\n{}",
            missing.len(),
            missing.join("\n")
        );
    });
}

#[test]
fn every_unbound_row_states_a_reason() {
    let (rows, known_rules) = manifest();
    let mut unexplained: Vec<String> = Vec::new();
    for row in &rows {
        if row.status == "not bound" && !known_rules.contains(&row.rule) {
            unexplained.push(format!("{}: rule `{}`", row.item, row.rule));
        }
    }
    assert!(
        unexplained.is_empty(),
        "{} rows are not bound and name no known reason:\n{}",
        unexplained.len(),
        unexplained.join("\n")
    );
}

#[test]
fn a_rule_the_manifest_binds_must_be_implemented() {
    let (rows, _rules) = manifest();
    let bound_rules: BTreeSet<&str> = rows
        .iter()
        .filter(|row| row.status == "bound")
        .map(|row| row.rule.as_str())
        .collect();
    let missing: Vec<&&str> = bound_rules
        .iter()
        .filter(|rule| !IMPLEMENTED_RULES.contains(rule))
        .collect();
    assert!(
        missing.is_empty(),
        "the manifest binds items of rules this test does not claim: {missing:?}. Either generate \
         the wrappers and add the rule to IMPLEMENTED_RULES, or move those rows to the manifest's \
         explicit-exception list with a reason."
    );
}
