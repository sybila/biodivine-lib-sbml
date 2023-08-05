extern crate proc_macro;
use proc_macro::TokenStream;
use quote::ToTokens;

/// Assert that the input is a struct with a single anonymous field.
fn check_single_field(ast: &syn::DeriveInput) {
    let fields = if let syn::Data::Struct(x) = &ast.data {
        &x.fields
    } else {
        panic!("This derive macro only works on struct types.")
    };
    if fields.len() != 1 {
        panic!("This derive macro only supports types with a single field.");
    }
    let field = fields.into_iter().next().unwrap();
    if field.ident.is_some() {
        panic!(
            "Unexpected field `{}` found.",
            field.ident.as_ref().unwrap()
        );
    }
}

/// Read a pair of identifier and type (`FnArg`) from the specified attribute.
fn read_name_type_pair(ast: &syn::DeriveInput, attr_name: &str) -> (String, String) {
    for attr in &ast.attrs {
        if attr.path.is_ident(attr_name) {
            let arg: syn::FnArg = attr.parse_args().unwrap();
            let syn::FnArg::Typed(arg) = arg else {
                panic!("Invalid typed argument.");
            };
            let name = format!("{}", arg.pat.to_token_stream());
            let r_type = format!("{}", arg.ty.to_token_stream());
            return (name, r_type);
        }
    }
    panic!("Missing `#[{}]`.", attr_name)
}

/// This procedural macro takes a declaration of a `T` `struct` type and derives the
/// following trait implementations for it:
///  - `XmlWrapper for T`
///  - `From<XmlElement> for T`
///  - `From<T> for XmlElement`
///
/// The only requirement is that the type must have an `xml` field whose type is `XmlElement`.
#[proc_macro_derive(XmlWrapper)]
pub fn derive_xml_wrapper(item: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(item).unwrap();
    let wrapper_type = ast.ident.to_string();
    check_single_field(&ast);

    let result = format!(
        r#"
        impl From<{wrapper_type}> for crate::xml::XmlElement {{
            fn from(value: {wrapper_type}) -> Self {{
                value.0
            }}
        }}

        impl From<crate::xml::XmlElement> for {wrapper_type} {{
            fn from(value: crate::xml::XmlElement) -> Self {{
                {wrapper_type}(value)
            }}
        }}

        impl crate::xml::XmlWrapper for {wrapper_type} {{
            fn as_xml(&self) -> &crate::xml::XmlElement {{
                &self.0
            }}
        }}
    "#
    );

    result.parse().unwrap()
}

/// This procedural macro derives the implementation of `XmlProperty` for a specific type
/// and attribute name. It requires that you annotate the struct with
/// `#[property_name("attribute_name")]` and `#[property_type(T)]`. Furthermore, the specified
/// `property_type` must implement `XmlPropertyType`, and the annotated type must have one
/// lifetime parameter, which refers to the lifetime of the underlying XML element.
///
/// Based on
/// https://stackoverflow.com/questions/56188700/how-do-i-make-my-custom-derive-macro-accept-trait-generic-parameters
/// https://stackoverflow.com/questions/71817839/accept-multiple-values-on-proc-macro-attribute
#[proc_macro_derive(XmlProperty, attributes(property))]
pub fn derive_xml_property(item: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(item).unwrap();
    let wrapper_type = ast.ident.to_string();
    let (property_name, property_type) = read_name_type_pair(&ast, "property");
    check_single_field(&ast);

    let result = format!(
        r#"
        impl<'a> {wrapper_type}<'a> {{
            pub fn for_element(element: &'a XmlElement) -> {wrapper_type}<'a> {{
                Self(element)
            }}
        }}

        impl crate::xml::XmlProperty<{property_type}> for {wrapper_type}<'_> {{
            fn element(&self) -> &XmlElement {{
                &self.0
            }}
            fn is_set(&self) -> bool {{
                crate::xml::generic_property::is_set(self.0, "{property_name}")
            }}
            fn read(&self) -> {property_type} {{
                crate::xml::generic_property::read(self.0, "{property_name}")
            }}
            fn read_checked(&self) -> Result<{property_type}, String> {{
                crate::xml::generic_property::read_checked(self.0, "{property_name}")
            }}
            fn read_raw(&self) -> Option<String> {{
                crate::xml::generic_property::read_raw(self.0, "{property_name}")
            }}
            fn clear(&self) {{
                crate::xml::generic_property::clear(self.0, "{property_name}");
            }}
            fn write(&self, value: &{property_type}) {{
                crate::xml::generic_property::write(self.0, "{property_name}", value);
            }}
            fn write_raw(&self, value: String) {{
                crate::xml::generic_property::write_raw(self.0, "{property_name}", value);
            }}
        }}
        "#
    );

    result.parse().unwrap()
}

/// Adds a "default" implementation for SBase which simply inherits all methods.
#[proc_macro_derive(SBase)]
pub fn derive_sbase(item: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(item).unwrap();
    let ttype = ast.ident.to_string();
    let result = format!(
        r#"
        impl crate::sbase::SBase for {ttype} {{}}
    "#
    );
    result.parse().unwrap()
}

/// This procedural macro derives the implementation of `XmlChild` for a specific type
/// and element name. It requires that you annotate the struct with
/// `#[child_name("attribute_name")]` and `#[child_type(T)]`. Furthermore, the specified
/// `child_type` must implement `XmlWrapper`, and the annotated type must have one
/// lifetime parameter, which refers to the lifetime of the underlying XML element.
///
#[proc_macro_derive(XmlChild, attributes(child))]
pub fn derive_xml_child(item: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(item).unwrap();
    let wrapper_type = ast.ident.to_string();
    let (child_name, child_type) = read_name_type_pair(&ast, "child");
    check_single_field(&ast);

    let result = format!(
        r#"
        impl<'a> {wrapper_type}<'a> {{
            pub fn for_element(element: &'a XmlElement) -> {wrapper_type}<'a> {{
                Self(element)
            }}
        }}

        impl crate::xml::XmlChild<{child_type}> for {wrapper_type}<'_> {{
            fn parent(&self) -> &crate::xml::XmlElement {{
                &self.0
            }}
            fn is_set(&self) -> bool {{
                crate::xml::generic_child::is_set(self.0, "{child_name}")
            }}
            fn clear(&self) {{
                crate::xml::generic_child::clear(self.0, "{child_name}");
            }}
            fn get(&self) -> {child_type} {{
                crate::xml::generic_child::get(self.0, "{child_name}")
            }}
            fn get_raw(&self) -> Option<crate::xml::XmlElement> {{
                crate::xml::generic_child::get_raw(self.0, "{child_name}")
            }}
            fn set(&self, element: {child_type}) -> Option<{child_type}> {{
                crate::xml::generic_child::set(self.0, "{child_name}", element)
            }}
            fn set_raw(&self, element: crate::xml::XmlElement) -> Option<crate::xml::XmlElement> {{
                crate::xml::generic_child::set_raw(self.0, "{child_name}", element)
            }}
        }}
        "#
    );

    result.parse().unwrap()
}
