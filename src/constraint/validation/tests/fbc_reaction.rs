use crate::Sbml;

#[test]
fn check_fbc_20716() {
    let doc = Sbml::read_path("test-inputs/test-fbc/strict_model.xml").unwrap();

    let issues = doc.validate().unwrap();

    assert_eq!(issues.len(), 1);
    assert_eq!(issues.first().unwrap().rule(), "fbc-20716")
}

#[test]
fn check_fbc_20705() {
    let doc = Sbml::read_path("test-inputs/test-fbc/test_fbc_20705.xml").unwrap();

    let issues = doc.validate().unwrap();

    println!("{:?}", issues);

    assert_eq!(issues.first().unwrap().rule(), "fbc-20705")
}

#[test]
fn check_fbc_20707() {
    let doc = Sbml::read_path("test-inputs/test-fbc/test_fbc_20707.xml").unwrap();

    let issues = doc.validate().unwrap();

    assert_eq!(issues.len(), 1);
    assert_eq!(issues.first().unwrap().rule(), "fbc-20707")
}
#[test]
fn check_fbc_20708() {
    let doc = Sbml::read_path("test-inputs/test-fbc/test_fbc_20708.xml").unwrap();

    let issues = doc.validate().unwrap();

    assert_eq!(issues.len(), 1);
    assert_eq!(issues.first().unwrap().rule(), "fbc-20708")
}
#[test]
fn check_fbc_20709() {
    let doc = Sbml::read_path("test-inputs/test-fbc/test_fbc_20709.xml").unwrap();

    let issues = doc.validate().unwrap();

    assert_eq!(issues.len(), 1);
    assert_eq!(issues.first().unwrap().rule(), "fbc-20709")
}
#[test]
fn check_fbc_207011() {
    let doc = Sbml::read_path("test-inputs/test-fbc/test_fbc_20711.xml").unwrap();

    let issues = doc.validate().unwrap();

    assert_eq!(issues.len(), 1);
    assert_eq!(issues.first().unwrap().rule(), "fbc-20711")
}
#[test]
fn check_fbc_207012() {
    let doc = Sbml::read_path("test-inputs/test-fbc/test_fbc_20712.xml").unwrap();

    let issues = doc.validate().unwrap();

    assert_eq!(issues.len(), 1);
    assert_eq!(issues.first().unwrap().rule(), "fbc-20712")
}
#[test]
fn check_fbc_207013() {
    let doc = Sbml::read_path("test-inputs/test-fbc/test_fbc_20713.xml").unwrap();

    let issues = doc.validate().unwrap();

    assert_eq!(issues.len(), 1);
    assert_eq!(issues.first().unwrap().rule(), "fbc-20713")
}
#[test]
fn check_fbc_207010() {
    let doc = Sbml::read_path("test-inputs/test-fbc/test_fbc_20710.xml").unwrap();

    let issues = doc.validate().unwrap();

    assert_eq!(issues.len(), 1);
    assert_eq!(issues.first().unwrap().rule(), "fbc-20710")
}

/// The rule ids a corpus document reports, or `None` when the corpus is not unpacked.
///
/// The corpus is downloaded rather than committed (see `VALIDATION.md`), so a test that reads one of
/// its documents has to be able to skip, which is the convention of the whole-corpus gates: it
/// prints why and returns. The documents used below are the SBML test suite's own evidence for the
/// rules they are named after, which is what makes them a better fixture than a hand-written model:
/// each of them is the case the specification's rule is about.
fn corpus_rules(name: &str) -> Option<Vec<String>> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("syntactic")
        .join(name);
    let source = match std::fs::read_to_string(&path) {
        Ok(source) => source,
        Err(_) => {
            println!(
                "skipping: the document {} is not unpacked (see VALIDATION.md)",
                path.display()
            );
            return None;
        }
    };
    let document =
        Sbml::read_str(&source).unwrap_or_else(|error| panic!("cannot read {name}: {error}"));
    Some(
        document
            .validate()
            .unwrap_or_else(|error| panic!("cannot validate {name}: {error}"))
            .into_iter()
            .map(|issue| issue.rule().to_string())
            .collect(),
    )
}

/// Rule fbc-20711 is about the value `INF`, not about either infinity.
///
/// The specification is explicit ("the value of the Parameter object referred to by the attribute
/// `fbc:lowerFluxBound` may not have the value `INF`"), and the suite has both directions: a pass
/// document whose lower bound is `-INF` (the most permissive bound there is) and two fail documents
/// whose lower bound is `INF`. A check without the sign test reports the pass document, which is what
/// this test pins.
#[test]
fn the_lower_flux_bound_rule_is_about_inf_and_not_about_either_infinity() {
    let Some(pass) = corpus_rules("fbc-20711/fbc-20711-pass-00-01-sev1-l3v1.xml") else {
        return;
    };
    assert!(
        !pass.contains(&"fbc-20711".to_string()),
        "a lower bound of -INF is permitted: {pass:?}"
    );

    // `-fail-01-02` is deliberately absent: it declares the *version 3* FBC namespace
    // (`.../fbc/version3`), which this crate does not model (it models `.../fbc/version2`), so its
    // FBC content is not read at all. That is a scope-out of the FBC package support, not a
    // behaviour of this rule, and it is recorded with the other package gaps.
    let Some(fail) = corpus_rules("fbc-20711/fbc-20711-fail-01-01-sev2-l3v1.xml") else {
        return;
    };
    assert!(
        fail.contains(&"fbc-20711".to_string()),
        "a lower bound of INF is not permitted: {fail:?}"
    );
}

/// The two flux-bound reference rules are separate rules about separate attributes: fbc-20705 is
/// about `fbc:lowerFluxBound` and fbc-20706 about `fbc:upperFluxBound`.
///
/// Both corpus documents name a parameter that the model does not define; they differ in which of
/// the two attributes does it, and each expects the rule of its own attribute.
#[test]
fn the_flux_bound_reference_rules_are_split_by_attribute() {
    let Some(lower) = corpus_rules("fbc-20705/fbc-20705-fail-01-01-sev2-l3v1.xml") else {
        return;
    };
    assert!(lower.contains(&"fbc-20705".to_string()), "{lower:?}");
    assert!(!lower.contains(&"fbc-20706".to_string()), "{lower:?}");

    let Some(upper) = corpus_rules("fbc-20706/fbc-20706-fail-01-01-sev2-l3v1.xml") else {
        return;
    };
    assert!(upper.contains(&"fbc-20706".to_string()), "{upper:?}");
    assert!(!upper.contains(&"fbc-20705".to_string()), "{upper:?}");
}

/// The three rules about the species references of a strict reaction are reported separately: the
/// `constant` attribute (fbc-20714), the finiteness of the `stoichiometry` (fbc-20715) and the
/// initial-assignment target (fbc-20716).
///
/// They used to share one report under fbc-20714, which turned the stoichiometry documents into a
/// report of a rule they do not break and hid the rule they do.
#[test]
fn the_strict_species_reference_rules_are_reported_separately() {
    let Some(non_constant) = corpus_rules("fbc-20714/fbc-20714-fail-01-01-sev2-l3v1.xml") else {
        return;
    };
    assert!(
        non_constant.contains(&"fbc-20714".to_string()),
        "{non_constant:?}"
    );
    assert!(
        !non_constant.contains(&"fbc-20715".to_string()),
        "{non_constant:?}"
    );

    for name in [
        "fbc-20715/fbc-20715-fail-01-01-sev2-l3v1.xml",
        "fbc-20715/fbc-20715-fail-01-02-sev2-l3v1.xml",
        "fbc-20715/fbc-20715-fail-01-03-sev2-l3v1.xml",
    ] {
        let Some(rules) = corpus_rules(name) else {
            return;
        };
        assert!(
            rules.contains(&"fbc-20715".to_string()),
            "a non-finite stoichiometry is fbc-20715: {name} reported {rules:?}"
        );
        assert!(
            !rules.contains(&"fbc-20714".to_string()),
            "the reference is constant, so fbc-20714 does not apply: {name} reported {rules:?}"
        );
    }
}
