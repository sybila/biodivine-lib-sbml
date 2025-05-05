use crate::xml::XmlDocument;
use crate::xml::XmlWrapper;
use biodivine_xml_doc::Element;
use pyo3::pyclass;
use pyo3_stub_gen_derive::gen_stub_pyclass;
use std::ops::DerefMut;
use std::sync::Arc;

/// An [XmlElement] maintains a single thread-safe reference to an [Element] of a [biodivine_xml_doc::Document].
///
/// Internally, this is achieved through a reference counted [std::sync::RwLock] (see [XmlDocument]).
///
/// Note that a lot of the useful functionality of [XmlElement] is actually implemented
/// through the [XmlWrapper] trait. The main difference is that [XmlWrapper] can be also
/// implemented for other types derived from [XmlElement].
#[derive(Clone, Debug)]
#[gen_stub_pyclass]
#[pyclass]
pub struct XmlElement {
    pub(super) document: XmlDocument,
    pub(super) element: Element,
}

impl PartialEq for XmlElement {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.document, &other.document) && self.element == other.element
    }
}

impl Eq for XmlElement {}

impl XmlElement {
    /// Wrap an existing [Element] as [XmlElement] in the context of the given [XmlDocument].
    pub fn new_raw(document: XmlDocument, element: Element) -> XmlElement {
        XmlElement { document, element }
    }

    /// Create a new empty [XmlElement] in the given [XmlDocument] with the given
    /// `name` and `namespace_prefix` + `namespace_url`. The element is created in a "detached"
    /// state.
    ///
    /// You can use empty string as namespace URL if you want to use the document
    /// namespace. Also, you can use empty string as namespace prefix if you want to
    pub fn new_quantified(
        document: XmlDocument,
        name: &str,
        namespace: (&str, &str),
    ) -> XmlElement {
        let element = {
            let mut doc = document.write().unwrap();
            Element::build(name)
                .prefix(namespace.0)
                .namespace_decl(namespace.0, namespace.1)
                .finish(doc.deref_mut())
        };
        XmlElement::new_raw(document, element)
    }

    /// Traverse the element tree towards the root element and return it.
    ///
    /// Note that if the element is detached, this will only reach the root of the detached
    /// group of elements.
    pub fn reachable_root(&self) -> XmlElement {
        let doc = self.document.read().unwrap();
        let mut element = self.element;
        while let Some(parent) = element.parent(&doc) {
            element = parent;
        }
        XmlElement::new_raw(self.document.clone(), element)
    }

    /// Returns the (first) root element of the associated [`XmlDocument`], even if this
    /// element itself is detached.
    ///
    /// This method can panic if the document root does not exist (e.g. the document is empty
    /// and only contains detached elements).
    pub fn document_root(&self) -> XmlElement {
        let doc = self.document.read().unwrap();
        let root = doc.root_element().unwrap();
        XmlElement::new_raw(self.document.clone(), root)
    }
}

/// Every [XmlElement] trivially implements [XmlWrapper] as well.
impl XmlWrapper for XmlElement {
    fn xml_element(&self) -> &XmlElement {
        self
    }

    unsafe fn unchecked_cast<T: XmlWrapper>(element: T) -> Self {
        element.xml_element().clone()
    }
}
