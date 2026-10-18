//! The rules an SBML package contributes to a document's validation.

use crate::cancellation::CancellationCheck;
use crate::validation::RuleSet;
use crate::{Sbml, SbmlIssue, SbmlResult};

/// The validation rules of an SBML *package*, and the checks that report them.
///
/// A package adds rules of its own, and this trait is how they take part in the validation of a
/// document: [`Sbml::validate_with`] runs the checks of every extension it is given, after the
/// checks of this crate, and merges their issues into the same report. Each issue carries the id,
/// the [kind](crate::RuleKind) and the [severity](crate::SbmlIssueSeverity) of the rule that produced it,
/// exactly as the issues of this crate do, so a caller cannot tell an injected rule from a built-in
/// one except by its id.
///
/// ```
/// use biodivine_lib_sbml::extension::ExtensionValidation;
/// use biodivine_lib_sbml::CancellationCheck;
/// use biodivine_lib_sbml::xml::{Document, OptionalXmlChild, XmlWrapper};
/// use biodivine_lib_sbml::{
///     RuleDescriptor, RuleKind, RuleSet, Sbml, SbmlIssue, SbmlIssueSeverity, SbmlResult,
///     URL_SBML_CORE,
/// };
///
/// /// The rules of the package: one rule of its own specification, and one diagnostic that no
/// /// specification owns.
/// struct MyExtensionRules {
///     rules: RuleSet,
/// }
///
/// impl Default for MyExtensionRules {
///     fn default() -> Self {
///         MyExtensionRules {
///             rules: RuleSet::new(vec![
///                 RuleDescriptor::new(
///                     "myext-10101",
///                     RuleKind::SpecRule,
///                     SbmlIssueSeverity::Error,
///                     "A model must not be empty.",
///                 ),
///                 RuleDescriptor::new(
///                     "myext-empty-list",
///                     RuleKind::LibraryLint,
///                     SbmlIssueSeverity::Info,
///                     "A list of the package is present but empty.",
///                 ),
///             ]),
///         }
///     }
/// }
///
/// impl ExtensionValidation for MyExtensionRules {
///     fn rules(&self) -> &RuleSet {
///         &self.rules
///     }
///
///     fn validate(
///         &self,
///         document: &Sbml,
///         issues: &mut Vec<SbmlIssue>,
///         check: &mut CancellationCheck,
///     ) -> SbmlResult<()> {
///         check.tick()?;
///         let Some(model) = document.model().get() else {
///             return Ok(());
///         };
///         if model.xml_element().child_elements().is_empty() {
///             issues.push(SbmlIssue::new_in(
///                 self.rules(),
///                 "myext-10101",
///                 &model,
///                 "The <model> element of this document is empty.",
///             ));
///         }
///         Ok(())
///     }
/// }
///
/// let document = Sbml::read_str(&format!(
///     r#"<sbml xmlns="{URL_SBML_CORE}" level="3" version="2"><model/></sbml>"#
/// ))?;
/// let extension = MyExtensionRules::default();
/// let issues = document.validate_with(&[&extension])?;
/// let issue = issues
///     .iter()
///     .find(|it| it.rule() == "myext-10101")
///     .expect("The package's rule reported the empty model.");
/// assert_eq!(issue.kind(), RuleKind::SpecRule);
/// assert_eq!(issue.severity(), SbmlIssueSeverity::Error);
///
/// // The rule set of the run - this crate's rules plus the package's - is what classifies the ids
/// // for a comparison against the SBML test suite.
/// let rules = RuleSet::CORE.with(extension.rules());
/// assert!(rules.is_spec_rule("myext-10101"));
/// assert!(!rules.is_spec_rule("myext-empty-list"));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # What a rule of a package is
///
/// The two kinds of issue this crate reports are a property of the rule, not of the caller
/// ([`RuleKind`](crate::RuleKind)), and a package's rules follow the same split:
///
///  * a rule **the package's own specification defines** - the package specifies it, its document
///    gives it an id and a severity - is a [`SpecRule`](crate::RuleKind::SpecRule), and its id is the
///    id the package specification gives it. The id has to have the shape of a specification rule id
///    (five digits, optionally prefixed by the package name, for example `fbc-21207`), because the
///    SBML test suite's expectation files are written in that vocabulary and the harness compares
///    against them.
///  * a diagnostic **no specification owns** - this library's opinion about the document rather than
///    a requirement of the package - is a [`LibraryLint`](crate::RuleKind::LibraryLint), its id is a
///    word of the package's choosing, and its severity is [`Info`](crate::SbmlIssueSeverity::Info) unless
///    the diagnostic means that a typed read of the document is impossible. A lint never takes part
///    in the comparison against the test suite, so it cannot make a valid document "fail".
///
/// The classification is data, and it is what the census filter of the test-suite harness asks about:
/// `RuleSet::CORE.with(extension.rules()).is_spec_rule(id)`. Reporting a rule of the package under a
/// lints' kind (or the other way round) would hide a real violation from the comparison or turn the
/// library's opinion into one, which is why the kind belongs in the descriptor.
///
/// # What an implementation may assume, and what it must do
///
///  * it is called **after** the checks of this crate, once per
///    [`Sbml::validate_with`] call, in the order the extensions are given in;
///  * `issues` is the report of the whole document, so it may add to it, and it must not remove
///    from it;
///  * the cancellation countdown of the call is shared: the implementation accounts for its own work
///    with [`CancellationCheck::tick`], so a request to stop a long validation is honoured inside the
///    package's walk as well. A package whose walk is bounded (a fixed number of elements) may decide
///    not to tick;
///  * the document it receives may be one the package is not declared in. Nothing in this crate
///    filters the extensions by the declarations of the document - the package decides what it
///    wants to check, and [`ExtensionCast`](crate::extension::ExtensionCast) is the mechanism that
///    asks whether the document declares it;
///  * the document may be one whose typed object model did not resolve (a level this crate does not
///    model); such a document is reported by the `unmodelled-level` lint of this crate, and a package
///    that wants to check it has to work on the raw XML.
pub trait ExtensionValidation {
    /// The rules of the package: the descriptors of every id an implementation of
    /// [`validate`](ExtensionValidation::validate) can report.
    ///
    /// The set is the package's own; the ids of this crate are in [`RuleSet::CORE`] and an issue of a
    /// core rule is reported with [`SbmlIssue::new`], while an issue of a rule of the set is reported
    /// with [`SbmlIssue::new_in`]. Combining the two sets (`RuleSet::CORE.with(..)`) is what a caller
    /// needs when it has to classify an id, for example for the test-suite comparison.
    fn rules(&self) -> &RuleSet;

    /// Checks the part of `document` this package owns.
    ///
    /// # Errors
    ///
    /// Returns [`SbmlError::Cancelled`](crate::SbmlError::Cancelled) when a cancellation trigger of
    /// the current thread has fired. A problem of the document is an issue (a value pushed into
    /// `issues`), not an error.
    fn validate(
        &self,
        document: &Sbml,
        issues: &mut Vec<SbmlIssue>,
        check: &mut CancellationCheck,
    ) -> SbmlResult<()>;
}
