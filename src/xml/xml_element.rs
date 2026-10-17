use crate::xml::XmlWrapper;
use biodivine_lib_xml_dom::xml_spec::Text;
use biodivine_lib_xml_dom::{Document, Element, Namespace, QualifiedName};
use std::collections::BTreeMap;

/// A thin, type-safe wrapper around a single [`Element`] of a [`Document`].
///
/// Internally, an `XmlElement` is just a handle to an element of a
/// [biodivine_lib_xml_dom] document. Such a handle is cheap to clone, thread-safe and
/// keeps the underlying element alive, so no external locking or reference counting is
/// necessary. This is a deliberate change from the previous design, which wrapped the
/// document in an `Arc<RwLock<..>>`.
///
/// The wrapper adds the small set of convenience operations that the SBML object model
/// needs on top of the raw [biodivine_lib_xml_dom] API. Access to the underlying element
/// is explicit through [`XmlElement::element`] and [`XmlElement::into_element`]; the
/// `XmlElement` intentionally does **not** dereference to [`Element`], so the set of
/// available operations is visible in one place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct XmlElement(Element);

impl XmlElement {
    /// Wrap an existing [biodivine_lib_xml_dom] element.
    pub fn new(element: Element) -> XmlElement {
        XmlElement(element)
    }

    /// Create a new, detached element with the given local `name` in the given `namespace`.
    ///
    /// The namespace is declared on the new element if it is not already in scope, so that the
    /// resulting element serializes correctly wherever it is later attached.
    ///
    /// # Panics
    ///
    /// Panics if `name` is not a valid XML name, or if `namespace` is not a valid XML namespace.
    /// The constant SBML namespaces are always valid.
    pub fn new_qualified(
        document: Document,
        name: &str,
        namespace: crate::constants::namespaces::Namespace,
    ) -> XmlElement {
        let qualified_name = crate::constants::namespaces::qualified_name(namespace, name)
            .expect("The provided element name is not a valid XML name.");
        let element = XmlElement(document.create_element(qualified_name));
        if let Some(namespace) = crate::constants::namespaces::to_xml_namespace(namespace) {
            element.ensure_namespace(&namespace);
        }
        element
    }

    /// A reference to the underlying [biodivine_lib_xml_dom] element.
    pub fn element(&self) -> &Element {
        &self.0
    }

    /// Consume this wrapper and return the underlying [biodivine_lib_xml_dom] element.
    pub fn into_element(self) -> Element {
        self.0
    }

    /// The document to which this element belongs.
    pub fn document(&self) -> Document {
        self.0.document()
    }

    /// The expanded (qualified) name of this element.
    pub fn qualified_name(&self) -> QualifiedName {
        self.0.qualified_name()
    }

    /// The local name of this element (without any namespace prefix).
    pub fn tag_name(&self) -> String {
        self.0.local_name().as_str().to_string()
    }

    /// The fully qualified name of this element, including the namespace prefix if one is
    /// attached to the name (e.g. `layout:speciesGlyph`).
    pub fn full_name(&self) -> String {
        self.0.qualified_name().to_string()
    }

    /// The URL of the namespace of this element, or an empty string if the element is in no
    /// namespace.
    pub fn namespace_url(&self) -> String {
        self.0
            .namespace()
            .map(|it| it.uri().to_string())
            .unwrap_or_default()
    }

    /// The attributes of this element, keyed by their expanded name.
    pub fn attributes(&self) -> BTreeMap<QualifiedName, Text> {
        self.0.attributes()
    }

    /// Returns `true` if this element has an attribute with the given local name, ignoring
    /// the namespace of the attribute.
    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes()
            .keys()
            .any(|it| it.local_name_str() == name)
    }

    /// Returns the value of the attribute with the given local name, ignoring the namespace
    /// of the attribute.
    pub fn get_attribute(&self, name: &str) -> Option<String> {
        self.attributes()
            .iter()
            .find(|(key, _value)| key.local_name_str() == name)
            .map(|(_key, value)| value.as_str().to_string())
    }

    /// The concatenated text content of this element and all its descendants.
    pub fn text_content(&self) -> String {
        let mut result = String::new();
        for node in self.0.descendants() {
            if let Some(text) = node.text() {
                result.push_str(text.as_str());
            }
        }
        result
    }

    /// The text content of this element with the surrounding whitespace removed.
    ///
    /// MathML payloads are routinely written on their own lines (`<ci>\n  x\n</ci>`), and the
    /// XML data model keeps that whitespace, so it has to be ignored where the text is
    /// *interpreted* as an SBML value (typically an identifier inside a `ci` or `bvar` element).
    /// The document keeps the original text: use [XmlElement::text_content] to read the payload
    /// verbatim.
    ///
    /// This is the XML Schema `collapse` facet as far as the SBML attribute types need it: only
    /// the whitespace *surrounding* the value is removed, because every SBML attribute type is a
    /// single token. A value whose meaning depends on whitespace *inside* it (a sequence of
    /// tokens such as the MathML `cn` payload) is not collapsed here.
    pub(crate) fn interpreted_text(&self) -> String {
        self.text_content().trim().to_string()
    }

    /// Replace the content of this element with a single text node containing `text`.
    ///
    /// # Panics
    ///
    /// Panics if `text` contains characters that cannot appear in an XML document.
    #[track_caller]
    pub fn set_text_content(&self, text: &str) {
        for child in self.0.children() {
            child.detach();
        }
        if !text.is_empty() {
            let text = self
                .0
                .document()
                .create_text(text)
                .expect("The provided text content is not valid XML.");
            self.0.append_child(text);
        }
    }

    /// The parent element of this element, if it has one.
    pub fn parent(&self) -> Option<XmlElement> {
        self.0
            .parent()
            .and_then(|it| it.as_element())
            .map(XmlElement)
    }

    /// The root element of the containing document, even if this element itself is detached.
    pub fn document_root(&self) -> Option<XmlElement> {
        self.0.document().root().map(XmlElement)
    }

    /// The immediate child elements of this element.
    pub fn child_elements(&self) -> Vec<XmlElement> {
        self.0
            .child_elements()
            .into_iter()
            .map(XmlElement)
            .collect()
    }

    /// A version of [`XmlElement::child_elements`] with an additional filter.
    pub fn child_elements_filtered<P: FnMut(&XmlElement) -> bool>(
        &self,
        mut predicate: P,
    ) -> Vec<XmlElement> {
        self.child_elements()
            .into_iter()
            .filter(|it| predicate(it))
            .collect()
    }

    /// The `index`-th child element of this element, ignoring text and comments.
    pub fn get_child_at(&self, index: usize) -> Option<XmlElement> {
        self.0
            .child_elements()
            .into_iter()
            .nth(index)
            .map(XmlElement)
    }

    /// All descendant elements of this element, in document order.
    pub fn recursive_child_elements(&self) -> Vec<XmlElement> {
        self.0
            .descendants()
            .into_iter()
            .filter_map(|it| it.as_element())
            .map(XmlElement)
            .collect()
    }

    /// A version of [`XmlElement::recursive_child_elements`] with an additional filter.
    pub fn recursive_child_elements_filtered<P: FnMut(&XmlElement) -> bool>(
        &self,
        mut predicate: P,
    ) -> Vec<XmlElement> {
        self.recursive_child_elements()
            .into_iter()
            .filter(|it| predicate(it))
            .collect()
    }

    /// Make sure that `namespace` is in scope for this element, declaring it on the element
    /// itself if it is not already visible.
    ///
    /// This is the "idiomatic namespace introduction" used when a new element or attribute
    /// is created in a namespace (for example an SBML package) that the surrounding document
    /// does not declare yet. Existing declarations are never rewritten or removed.
    pub fn ensure_namespace(&self, namespace: &Namespace) {
        if !self.is_namespace_in_scope(namespace) {
            self.0.declare_namespace(namespace.clone());
        }
    }

    /// Returns `true` if the effective in-scope binding for the prefix of `namespace` binds
    /// that prefix to the URI of `namespace`.
    fn is_namespace_in_scope(&self, namespace: &Namespace) -> bool {
        let wanted_prefix = namespace.prefix().cloned();
        match self
            .0
            .namespaces_in_scope()
            .into_iter()
            .find(|(prefix, _binding)| *prefix == wanted_prefix)
        {
            Some((_prefix, Some(binding))) => binding.has_same_uri(namespace),
            _ => false,
        }
    }
}

/// Every [XmlElement] trivially implements [XmlWrapper] as well.
impl XmlWrapper for XmlElement {
    fn xml_element(&self) -> &XmlElement {
        self
    }
}
