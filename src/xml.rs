use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use xml_doc::{Document, Element};

/// A type alias which defines `XmlDocument` as a `xml_doc::Document` object
/// that is wrapped in a reference-counted read-write lock. This makes the
/// document (1) thread safe for parallel computing and (2) memory safe outside
/// of Rust's borrow checker capabilities.
pub type XmlDocument = Arc<RwLock<Document>>;

/// An [XmlElement] maintains a single thread-safe reference to an [Element] of a [Document].
///
/// Internally, this is achieved through a reference counted [RwLock] (see [XmlDocument]).
#[derive(Clone, Debug)]
pub struct XmlElement {
    document: XmlDocument,
    element: Element,
}

impl XmlElement {
    /// Create a new [XmlElement] from an existing [XmlDocument] and [Element].
    pub fn new(document: XmlDocument, element: Element) -> XmlElement {
        XmlElement { document, element }
    }

    /// Create a new [XmlElement] from an existing [Document] and [Element].
    ///
    /// Note that this method takes the ownership of the whole XML [Document] and wraps it into
    /// a thread-safe [XmlDocument]. You can then create new [XmlElement] objects for this document
    /// using [XmlElement::derive].
    pub fn build(document: Document, element: Element) -> XmlElement {
        let document = Arc::new(RwLock::new(document));
        Self::new(document, element)
    }

    /// Create a new [XmlElement] which has the same underlying [XmlDocument] as *this*
    /// [XmlElement], but references a different [Element].
    pub fn derive(&self, element: Element) -> XmlElement {
        XmlElement {
            document: self.document.clone(),
            element,
        }
    }

    /// Obtain a read-only reference to the underlying [Document].
    pub fn read_doc(&self) -> RwLockReadGuard<Document> {
        // Error handling note: In general, lock access will fail only when some other part
        // of the program performed an incorrect unsafe action (e.g. double release of the
        // same lock guard). As such, it is generally ok to panic here, because at that point
        // the whole document might be corrupted and we have no way to recover.
        self.document
            .read()
            .expect("Underlying document lock is corrupted. Cannot recover.")
    }

    /// Obtain a writeable reference to the underlying [Document].
    pub fn write_doc(&self) -> RwLockWriteGuard<Document> {
        // See [Self::read_doc] for error handling notes.
        self.document
            .write()
            .expect("Underlying document lock is corrupted. Cannot recover.")
    }

    /// Get the [Element] instance of this [XmlElement].
    pub fn element(&self) -> Element {
        self.element
    }
}

/// This trait is implemented by types that wrap the contents of a specific [XmlElement] into
/// a different interface (usually more type-safe and domain specific interface).
///
/// For convenience, the trait also provides parts of the [XmlElement] API that are most commonly
/// used so that you don't have to run [XmlWrapper::as_xml] every time you want to perform some
/// operation on the underlying document.
///
/// Note that roughly the same functionality can be achieved through the standard generic [Deref]
/// trait. However, it is not recommended to implement [Deref] for types which are not
/// smart pointers, i.e. types that have other functionality beyond just pointing to
/// the underlying data in some non-standard way.
pub trait XmlWrapper {
    /// Obtain a reference to the underlying [XmlElement].
    fn as_xml(&self) -> &XmlElement;

    /// See [XmlElement::element].
    fn element(&self) -> Element {
        self.as_xml().element()
    }

    /// See [XmlElement::read_doc].
    fn read_doc(&self) -> RwLockReadGuard<Document> {
        self.as_xml().read_doc()
    }

    /// See [XmlElement::write_doc].
    fn write_doc(&self) -> RwLockWriteGuard<Document> {
        self.as_xml().write_doc()
    }
}
