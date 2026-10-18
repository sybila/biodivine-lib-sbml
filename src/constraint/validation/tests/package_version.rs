//! The version of a *package* a document declares, and what the read path does with it.
//!
//! A package has one namespace URL per published version (`FBC_NAMESPACE_URLS`), while the crate's
//! accessors, tables and requirements are written against one version of it. These tests pin the two
//! halves of that split:
//!
//!  * **reading follows the document**: a property read resolves the package to the namespace the
//!    document binds, so the FBC attributes of a Version 1 document are found even though the crate
//!    writes the Version 2 URL (`FbcSpecies::formula`, and the same resolution in the child layer);
//!  * **validation follows the model**: the package checks of this crate are the checks of the
//!    version its tables describe, and a document that declares another published version of the
//!    package is not validated against them ([`Sbml::declares_modelled_package`]).
use crate::constraint::FbcSpecies;
use crate::xml::{OptionalXmlChild, OptionalXmlProperty, XmlSubtype};
use crate::{Sbml, FBC_NAMESPACE_URLS, URL_SBML_CORE};

/// A document that declares the FBC package in `version` under the prefix `prefix`, and carries one
/// species with an FBC formula, in the Level 3 Version 1 core namespace the corpus uses for package
/// documents.
fn document_with_fbc_version(version: &str, extra_model_attributes: &str) -> String {
    fbc_document(version, "fbc", extra_model_attributes)
}

/// The same document, with the prefix of the package declaration given explicitly.
fn fbc_document(version: &str, prefix: &str, extra_model_attributes: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
           <sbml xmlns="{URL_SBML_CORE}" xmlns:{prefix}="{version}" level="3" version="1"
                 {prefix}:required="false">
             <model id="m" {extra_model_attributes}>
               <listOfCompartments><compartment id="c" size="1" constant="true"/></listOfCompartments>
               <listOfSpecies>
                 <species id="s" compartment="c" initialAmount="1" constant="false"
                          {prefix}:charge="2" {prefix}:chemicalFormula="C6H12O6"/>
               </listOfSpecies>
             </model>
           </sbml>"#
    )
}

#[test]
fn an_fbc_property_is_read_in_the_namespace_the_document_binds() {
    // The accessors are written against `NS_FBC` (the Version 2 URL). A Version 1 document keeps the
    // same attribute in the Version 1 URL, so a lookup in the written URL finds nothing; the read
    // path resolves the package through the document instead.
    for version in FBC_NAMESPACE_URLS {
        let document = Sbml::read_str(&document_with_fbc_version(version, "")).unwrap();
        let model = document.model().get().unwrap();
        let species = model.species().get().unwrap().get(0);
        let species = FbcSpecies::try_cast_from_super(&species)
            .expect("A species of a document that declares FBC is an FBC species.");

        assert_eq!(
            species.formula().get(),
            Some("C6H12O6".to_string()),
            "The formula of a document that binds {version} must be readable."
        );
        assert_eq!(species.charge().get(), Some(2), "charge of {version}");
    }
}

#[test]
fn a_document_that_binds_another_package_version_is_not_checked_against_these_tables() {
    // `model@fbc:strict` is required by FBC Version 2 and by neither Version 1 nor Version 3, and
    // the crate's table states the Version 2 requirement. A document that uses another version is
    // therefore not reported for it; doing so was a measured regression (65 unexpected events,
    // `notes/g42-report.md`).
    let modelled = Sbml::read_str(&document_with_fbc_version(FBC_NAMESPACE_URLS[1], "")).unwrap();
    assert!(modelled.declares_modelled_package(crate::NS_FBC));
    let reported = issues_of(&modelled);
    assert!(
        reported.contains(&"20222".to_string()),
        "The modelled version does require the attribute: {reported:?}"
    );

    for version in [FBC_NAMESPACE_URLS[0], FBC_NAMESPACE_URLS[2]] {
        let document = Sbml::read_str(&document_with_fbc_version(version, "")).unwrap();
        // The package is declared all the same - it is just another version of it.
        assert!(document.declared_package_prefix(crate::NS_FBC).is_ok());
        assert!(!document.declares_modelled_package(crate::NS_FBC));
        let reported = issues_of(&document);
        assert!(
            !reported.contains(&"20222".to_string()),
            "A document that binds {version} must not be required to carry the Version 2 \
             attribute: {reported:?}"
        );
        assert!(
            !reported.iter().any(|rule| rule.starts_with("fbc-")),
            "The FBC checks of this crate are the Version 2 checks: {reported:?}"
        );
    }
}

#[test]
fn the_prefix_of_the_declaration_is_the_documents_own() {
    // The declaration may use any prefix; the read path is namespace-aware, so the reported prefix
    // is what the document wrote and the checks are the same.
    let document = Sbml::read_str(&fbc_document(FBC_NAMESPACE_URLS[1], "flux", "")).unwrap();
    // The document binds the *Version 2* URL under `flux`, so `fbc` itself is not the declared
    // prefix: this crate validates the packaged version only under its conventional prefix.
    assert_eq!(
        document.declared_package_prefix(crate::NS_FBC),
        Ok("flux".to_string())
    );
    assert!(!document.declares_modelled_package(crate::NS_FBC));
    let model = document.model().get().unwrap();
    let species = model.species().get().unwrap().get(0);
    let species = FbcSpecies::try_cast_from_super(&species).unwrap();
    // Reading still finds the attribute: the resolution compares URLs, not prefixes.
    assert_eq!(species.formula().get(), Some("C6H12O6".to_string()));
}

/// The rule ids of a document's issues, for the tests that ask about a rule.
fn issues_of(document: &Sbml) -> Vec<String> {
    document
        .validate()
        .expect("The tests register no cancellation trigger.")
        .into_iter()
        .map(|issue| issue.rule().to_string())
        .collect()
}
