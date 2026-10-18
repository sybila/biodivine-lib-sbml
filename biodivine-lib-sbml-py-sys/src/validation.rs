//! The validation report: issues, rule descriptors and rule sets.
//!
//! The Rust API reports validation through values ([`SbmlIssue`], [`RuleSet`], [`RuleDescriptor`]),
//! and those values are what this module binds. The three enumerations they use
//! ([`RuleKind`](biodivine_lib_sbml::RuleKind), [`SbmlIssueSeverity`], and the `RuleKind` tags of
//! the classes below) become Python classes with the variants as attributes, so a Python caller
//! compares `issue.severity == SbmlIssueSeverity.ERROR` rather than a magic string.

use biodivine_lib_sbml::xml::XmlElement;
use biodivine_lib_sbml::{RuleDescriptor, RuleKind, RuleSet, SbmlIssue, SbmlIssueSeverity};
use pyo3::prelude::*;
use pyo3::types::PyType;

use crate::element::PyElement;
use crate::error::value_err;

/// What an issue means: a rule of the SBML standard, or a diagnostic of this library.
#[pyclass(
    name = "RuleKind",
    module = "biodivine_sys.sbml",
    eq,
    frozen,
    from_py_object
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PyRuleKind {
    pub(crate) inner: RuleKind,
}

#[pymethods]
impl PyRuleKind {
    /// A rule of the SBML specification, identified by the specification's rule id.
    #[classattr]
    #[pyo3(name = "SPEC_RULE")]
    fn spec_rule(py: Python<'_>) -> PyResult<Py<PyRuleKind>> {
        Py::new(
            py,
            PyRuleKind {
                inner: RuleKind::SpecRule,
            },
        )
    }

    /// A diagnostic this library raises on its own initiative (a "library lint").
    #[classattr]
    #[pyo3(name = "LIBRARY_LINT")]
    fn library_lint(py: Python<'_>) -> PyResult<Py<PyRuleKind>> {
        Py::new(
            py,
            PyRuleKind {
                inner: RuleKind::LibraryLint,
            },
        )
    }

    /// `True` for a rule of the SBML specification, `False` for a library lint.
    #[getter]
    fn is_spec_rule(&self) -> bool {
        self.inner == RuleKind::SpecRule
    }

    fn __repr__(&self) -> String {
        format!("RuleKind.{}", self.name())
    }

    /// The name of this kind, as it appears in `repr`.
    fn name(&self) -> &'static str {
        match self.inner {
            RuleKind::SpecRule => "SPEC_RULE",
            RuleKind::LibraryLint => "LIBRARY_LINT",
        }
    }
}

/// How serious an issue is.
#[pyclass(
    name = "SbmlIssueSeverity",
    module = "biodivine_sys.sbml",
    eq,
    frozen,
    from_py_object
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PyIssueSeverity {
    pub(crate) inner: SbmlIssueSeverity,
}

#[pymethods]
impl PyIssueSeverity {
    /// An issue that makes the document impossible to read correctly.
    #[classattr]
    #[pyo3(name = "ERROR")]
    fn error(py: Python<'_>) -> PyResult<Py<PyIssueSeverity>> {
        Py::new(
            py,
            PyIssueSeverity {
                inner: SbmlIssueSeverity::Error,
            },
        )
    }

    /// A possible error that does not necessarily make the document invalid.
    #[classattr]
    #[pyo3(name = "WARNING")]
    fn warning(py: Python<'_>) -> PyResult<Py<PyIssueSeverity>> {
        Py::new(
            py,
            PyIssueSeverity {
                inner: SbmlIssueSeverity::Warning,
            },
        )
    }

    /// A suggestion that does not represent a significant issue.
    #[classattr]
    #[pyo3(name = "INFO")]
    fn info(py: Python<'_>) -> PyResult<Py<PyIssueSeverity>> {
        Py::new(
            py,
            PyIssueSeverity {
                inner: SbmlIssueSeverity::Info,
            },
        )
    }

    fn __repr__(&self) -> String {
        format!("SbmlIssueSeverity.{}", self.name())
    }

    /// The name of this severity, as it appears in `repr`.
    fn name(&self) -> &'static str {
        match self.inner {
            SbmlIssueSeverity::Error => "ERROR",
            SbmlIssueSeverity::Warning => "WARNING",
            SbmlIssueSeverity::Info => "INFO",
        }
    }
}

/// One finding of a validation pass.
#[pyclass(
    name = "SbmlIssue",
    module = "biodivine_sys.sbml",
    frozen,
    from_py_object
)]
#[derive(Debug, Clone)]
pub struct PySbmlIssue {
    pub(crate) inner: SbmlIssue,
}

#[pymethods]
impl PySbmlIssue {
    /// Creates an issue of a rule of the core rule set.
    ///
    /// The kind and the severity come from the rule's descriptor, so an issue cannot be reported
    /// with a severity its rule does not have.
    ///
    /// Raises:
    ///     ValueError: if the rule id is not in `RuleSet.CORE`.
    #[staticmethod]
    #[pyo3(signature = (rule, element, message))]
    fn new(rule: &str, element: &PyElement, message: &str) -> PyResult<PySbmlIssue> {
        let rules = RuleSet::CORE;
        if rules.get(rule).is_none() {
            return Err(value_err(format!(
                "the core rule set has no rule `{rule}`; pass a rule set that describes it"
            )));
        }
        // `XmlElement` is itself a wrapper, so the raw element can carry the issue.
        let element = XmlElement::new(element.rust_element().clone());
        Ok(PySbmlIssue {
            inner: SbmlIssue::new_in(rules, rule, &element, message),
        })
    }

    /// Creates an issue that `rules` attributes to `rule`.
    ///
    /// This is the constructor for rules that are not in the core rule set: an SBML extension
    /// describes its rules with a `RuleSet` and reports through this method.
    ///
    /// Raises:
    ///     ValueError: if `rules` does not describe `rule`.
    #[staticmethod]
    #[pyo3(signature = (rules, rule, element, message))]
    fn new_in(
        rules: &PyRuleSet,
        rule: &str,
        element: &PyElement,
        message: &str,
    ) -> PyResult<PySbmlIssue> {
        if rules.inner.get(rule).is_none() {
            return Err(value_err(format!(
                "the rule set does not describe the rule `{rule}`"
            )));
        }
        let element = XmlElement::new(element.rust_element().clone());
        Ok(PySbmlIssue {
            inner: SbmlIssue::new_in(&rules.inner, rule, &element, message),
        })
    }

    /// The id of the rule that found the issue.
    ///
    /// It is a rule id of the SBML specification for a specification rule, and a word such as
    /// `not-validated` or `unused-namespace` for a library lint.
    #[getter]
    fn rule(&self) -> &'static str {
        self.inner.rule()
    }

    /// Whether the issue is a specification rule or a library lint.
    #[getter]
    fn kind(&self) -> PyRuleKind {
        PyRuleKind {
            inner: self.inner.kind(),
        }
    }

    /// The severity the rule assigns to the issue.
    #[getter]
    fn severity(&self) -> PyIssueSeverity {
        PyIssueSeverity {
            inner: self.inner.severity(),
        }
    }

    /// A human-readable description of the problem.
    #[getter]
    fn message(&self) -> &str {
        self.inner.message()
    }

    /// The XML element the problem was found on.
    #[getter]
    fn element(&self) -> PyElement {
        PyElement::from(self.inner.element().clone())
    }

    /// `True` when the issue says that a part of the document was **not** validated, because its
    /// type check failed and the walk could not descend into it.
    ///
    /// A report without such an issue means that every object of the document was walked.
    fn is_not_validated(&self) -> bool {
        self.inner.is_not_validated()
    }

    fn __repr__(&self) -> String {
        format!(
            "SbmlIssue(rule={:?}, severity={}, message={:?})",
            self.inner.rule(),
            PyIssueSeverity {
                inner: self.inner.severity()
            }
            .name(),
            self.inner.message()
        )
    }

    fn __str__(&self) -> String {
        format!("[{}] {}", self.inner.rule(), self.inner.message())
    }
}

/// The metadata of one rule: its id, kind, severity and a one-line summary.
#[pyclass(
    name = "RuleDescriptor",
    module = "biodivine_sys.sbml",
    frozen,
    from_py_object
)]
#[derive(Debug, Clone, Copy)]
pub struct PyRuleDescriptor {
    pub(crate) inner: RuleDescriptor,
}

#[pymethods]
impl PyRuleDescriptor {
    /// Creates a descriptor from its parts (the Rust constructor).
    #[staticmethod]
    #[pyo3(signature = (id, kind, severity, summary))]
    fn new(
        id: String,
        kind: &PyRuleKind,
        severity: &PyIssueSeverity,
        summary: String,
    ) -> PyResult<PyRuleDescriptor> {
        // The Rust constructor takes `&'static str`s, and a Python string is not one: the descriptor
        // therefore interns the two strings for the lifetime of the process. A rule descriptor is
        // created once, so this is a bounded, deliberate leak of two short strings.
        let id: &'static str = Box::leak(id.into_boxed_str());
        let summary: &'static str = Box::leak(summary.into_boxed_str());
        let _ = &kind.inner;
        Ok(PyRuleDescriptor {
            inner: RuleDescriptor::new(id, kind.inner, severity.inner, summary),
        })
    }

    /// The rule id.
    #[getter]
    fn id(&self) -> &'static str {
        self.inner.id
    }

    /// Whether the rule is a specification rule or a library lint.
    #[getter]
    fn kind(&self) -> PyRuleKind {
        PyRuleKind {
            inner: self.inner.kind,
        }
    }

    /// The severity of every issue this rule produces.
    #[getter]
    fn severity(&self) -> PyIssueSeverity {
        PyIssueSeverity {
            inner: self.inner.severity,
        }
    }

    /// A one-line summary of what the rule requires.
    #[getter]
    fn summary(&self) -> &'static str {
        self.inner.summary
    }

    fn __repr__(&self) -> String {
        format!(
            "RuleDescriptor(id={:?}, kind={}, severity={})",
            self.inner.id,
            PyRuleKind {
                inner: self.inner.kind
            }
            .name(),
            PyIssueSeverity {
                inner: self.inner.severity
            }
            .name()
        )
    }
}

/// A set of rule descriptors, looked up by rule id.
#[pyclass(
    name = "RuleSet",
    module = "biodivine_sys.sbml",
    frozen,
    from_py_object
)]
#[derive(Debug, Clone)]
pub struct PyRuleSet {
    pub(crate) inner: RuleSet,
}

#[pymethods]
impl PyRuleSet {
    /// The rule table of this library: every rule id it can report.
    #[classattr]
    #[pyo3(name = "CORE")]
    fn core(py: Python<'_>) -> PyResult<Py<PyRuleSet>> {
        Py::new(
            py,
            PyRuleSet {
                inner: RuleSet::CORE.clone(),
            },
        )
    }

    /// Builds a rule set from a list of descriptors.
    #[new]
    fn create(rules: Vec<PyRuleDescriptor>) -> Self {
        PyRuleSet {
            inner: RuleSet::new(rules.into_iter().map(|it| it.inner).collect()),
        }
    }

    /// Builds a rule set from a list of descriptors (the Rust constructor).
    #[staticmethod]
    fn new(rules: Vec<PyRuleDescriptor>) -> PyRuleSet {
        PyRuleSet::create(rules)
    }

    /// The number of rules in this set (the Rust accessor; `len(rule_set)` is the same).
    fn len(&self) -> usize {
        self.inner.len()
    }

    /// Whether this set describes no rules.
    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// The descriptor of `rule_id`, or `None` when this set does not describe it.
    fn get(&self, rule_id: &str) -> Option<PyRuleDescriptor> {
        self.inner
            .get(rule_id)
            .map(|inner| PyRuleDescriptor { inner: *inner })
    }

    /// Whether `rule_id` is a rule of the SBML specification.
    fn is_spec_rule(&self, rule_id: &str) -> bool {
        self.inner.is_spec_rule(rule_id)
    }

    /// Every descriptor of this set, in declaration order.
    fn rules(&self) -> Vec<PyRuleDescriptor> {
        self.inner
            .rules()
            .iter()
            .map(|inner| PyRuleDescriptor { inner: *inner })
            .collect()
    }

    /// This set followed by the rules of `other` that it does not already contain.
    fn with(&self, other: &PyRuleSet) -> PyRuleSet {
        PyRuleSet {
            inner: self.inner.with(&other.inner),
        }
    }

    fn __len__(&self) -> usize {
        self.inner.len()
    }

    fn __repr__(&self) -> String {
        format!("RuleSet({} rules)", self.inner.len())
    }
}

/// Registers the validation classes, which have no module-level functions of their own.
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    let _ = module.py().get_type::<PyType>();
    Ok(())
}
