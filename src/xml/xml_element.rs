use crate::xml::XmlDocument;
use crate::xml::XmlWrapper;
use std::ops::DerefMut;
use std::sync::Arc;
use xml_doc::Element;

/// An [XmlElement] maintains a single thread-safe reference to an [Element] of a [Document].
///
/// Internally, this is achieved through a reference counted [RwLock] (see [XmlDocument]).
///
/// Note that a lot of the useful functionality of [XmlElement] is actually implemented
/// through the [XmlWrapper] trait. The main difference is that [XmlWrapper] can be also
/// implemented for other types derived from [XmlElement].
#[derive(Clone, Debug)]
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
