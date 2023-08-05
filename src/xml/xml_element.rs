use crate::xml::XmlDocument;
use crate::xml::XmlWrapper;
use std::sync::{Arc, RwLock};
use xml_doc::{Document, Element};

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

impl XmlElement {
    /// Create a new [XmlElement] from an existing [XmlDocument] and [Element].
    pub fn new(document: XmlDocument, element: Element) -> XmlElement {
        XmlElement { document, element }
    }

    /// Create a new [XmlElement] from an existing [Document] and [Element].
    ///
    /// Note that this method takes the ownership of the whole XML [Document] and wraps it
    /// into a thread-safe [XmlDocument]. However, the resulting document can be accessed
    /// through [XmlWrapper::document].
    pub fn build(document: Document, element: Element) -> XmlElement {
        let document = Arc::new(RwLock::new(document));
        Self::new(document, element)
    }
}

/// Every [XmlElement] trivially implements [XmlWrapper] as well.
impl XmlWrapper for XmlElement {
    fn as_xml(&self) -> &XmlElement {
        self
    }
}
