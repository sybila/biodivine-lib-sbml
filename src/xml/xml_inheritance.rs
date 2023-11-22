use crate::xml::XmlWrapper;

/// Implemented by [XmlWrapper] instances that represent a variant of a more generic `Super` type.
///
/// In general, we expect the classic OOP substitution principle to hold. That is, every instance
/// of a sub-type can be used as a valid instance of the super type. However, casting a super type
/// into a sub-type involves a runtime check and can fail.
pub trait XmlSubtype<Super: XmlSupertype>: XmlWrapper {
    /// Try to cast the value of `Super` type as `Self`.
    ///
    /// Typically, this checks the name of the XML tag, but in som cases could also
    /// involve other properties of the XML tag.
    fn cast_from_super_checked(value: &Super) -> Option<Self>;

    /// Create a value of `Self` from the provided `Super` type.
    /// Panics when the cast is not successful.
    fn cast_from_super(value: &Super) -> Self {
        Self::cast_from_super_checked(value).unwrap_or_else(|| {
            panic!(
                "Cannot cast element of type `{}` as type `{}`.",
                std::any::type_name::<Super>(),
                std::any::type_name::<Self>()
            );
        })
    }

    /// Cast the value of this subtype as the `Super` type.
    ///
    /// This cast should be always valid, hence we can provide a default implementation.
    fn upcast(&self) -> Super {
        // Safety follows from the "contracts" prescribed for implementations of `XmlSubtype`.
        unsafe { Super::unchecked_cast(self.xml_element().clone()) }
    }
}

/// A counterpart for [XmlSubtype]. It provides default implementations for various utility
/// methods that are potentially more idiomatic than calling [XmlSubtype] directly.
///
/// Unfortunately, Rust does not allow us to provide a blanket implementation
pub trait XmlSupertype: XmlWrapper {
    /// Try to cast the value of `Self` as the value of `Sub` type. The `Sub` type must
    /// implement the appropriate [XmlSubtype] trait.
    ///
    /// See also [XmlSubtype::cast_from_super_checked].
    fn downcast_checked<Sub: XmlSubtype<Self>>(&self) -> Option<Sub> {
        Sub::cast_from_super_checked(self)
    }

    /// Cast the value of `Self` as the value of `Sub` type.
    /// Panics when the cast is unsuccessful.
    ///
    /// See also [XmlSubtype::cast_from_super].
    fn downcast<Sub: XmlSubtype<Self>>(&self) -> Sub {
        Sub::cast_from_super(self)
    }

    /// Cast the value of `Sub` as `Self`. This cast always succeeds.
    fn cast_from_sub<Sub: XmlSubtype<Self>>(value: &Sub) -> Self {
        value.upcast()
    }

    /// Returns `true` if `self` is of type `Sub`, i.e. it can be safely casted to `Sub`.
    fn is_instance<Sub: XmlSubtype<Self>>(&self) -> bool {
        self.downcast_checked::<Sub>().is_some()
    }
}

/// A "helper" trait that provides a default implementation for [XmlSubtype] as long as the
/// type is only determined by the name of the XML tag and the name is known at compile time.
pub trait XmlNamedSubtype<Super: XmlSupertype>: XmlSubtype<Super> {
    fn expected_tag_name() -> &'static str;
}

impl<Super: XmlSupertype, Sub: XmlNamedSubtype<Super>> XmlSubtype<Super> for Sub {
    fn cast_from_super_checked(value: &Super) -> Option<Self> {
        if value.tag_name() == Self::expected_tag_name() {
            unsafe { Some(Self::unchecked_cast(value.xml_element().clone())) }
        } else {
            None
        }
    }
}
