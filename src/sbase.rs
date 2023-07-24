use crate::xml::XmlWrapper;
use std::ops::{Deref, DerefMut};

/// Abstract class SBase that is the parent of most of the elements in SBML.
/// Thus there is no need to implement concrete structure.
pub trait SBase {
    fn get_id(&self) -> Option<String>;
    fn get_name(&self) -> Option<String>;
    fn get_metaid(&self) -> Option<String>;
    fn get_sboterm(&self) -> Option<String>;
    fn get_notes(&self) -> Option<String>;
    fn get_annotation(&self) -> Option<String>;
    fn set_id(&self, value: String) -> ();
    fn set_name(&self, value: String) -> ();
    fn set_metaid(&self, value: String) -> ();
    fn set_sboterm(&self, value: String) -> ();
    fn set_notes(&self, value: String) -> ();
    fn set_annotation(&self, value: String) -> ();
}

/// A trait implemented by types that should implement [SBase] using the default functionality
/// provided by the [XmlWrapper] trait.
///
/// This is a so-called "marker trait". On its own, it does nothing. However, it is used as
/// a "marker" for the compiler to indicate that we want something to only hold for types
/// where we explicitly include this trait.
///
/// The trait itself does not need to be public as long as [SBase] itself is public. However,
/// we could make it public if we wanted to enable other libraries to use the default [SBase]
/// implementation (e.g. if we wanted to implement SBML extensions as separate libraries and still
/// allow them to implement [SBase] the "default" way).
pub trait SBaseDefault {}

/// A generic "default" implementation of [SBase] for any type that implements both
/// [XmlWrapper] and [SBaseDefault].
impl<T: SBaseDefault + XmlWrapper> SBase for T {
    fn get_id(&self) -> Option<String> {
        let doc = self.read_doc();
        // Unfortunately, here the reference returned by the `attribute` function is only
        // valid for as long as the `xml` document is locked. Hence we can't return it,
        // because once this function completes, the lock is released and the reference becomes
        // unsafe to access. We thus have to copy the contents of the string using `to_string`.
        self.element()
            .attribute(doc.deref(), "id")
            .map(|it| it.to_string())
    }

    fn get_name(&self) -> Option<String> {
        let doc = self.read_doc();
        self.element()
            .attribute(doc.deref(), "name")
            .map(|it| it.to_string())
    }

    fn get_metaid(&self) -> Option<String> {
        let doc = self.read_doc();
        self.element()
            .attribute(doc.deref(), "metaid")
            .map(|it| it.to_string())
    }

    fn get_sboterm(&self) -> Option<String> {
        todo!()
    }

    fn get_notes(&self) -> Option<String> {
        todo!()
    }

    fn get_annotation(&self) -> Option<String> {
        todo!()
    }

    fn set_id(&self, value: String) -> () {
        let mut doc = self.write_doc();
        self.element().set_attribute(doc.deref_mut(), "id", value);
    }

    fn set_name(&self, value: String) -> () {
        todo!()
    }

    fn set_metaid(&self, value: String) -> () {
        todo!()
    }

    fn set_sboterm(&self, value: String) -> () {
        todo!()
    }

    fn set_notes(&self, value: String) -> () {
        todo!()
    }

    fn set_annotation(&self, value: String) -> () {
        todo!()
    }
}
