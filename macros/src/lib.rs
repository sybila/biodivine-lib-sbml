extern crate proc_macro;
use proc_macro::TokenStream;
use std::io::Write;
use syn::__private::ToTokens;

// TODO: Large parts of the macro implementation could be moved to some helper methods.

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
    let fields = if let syn::Data::Struct(x) = ast.data {
        x.fields
    } else {
        panic!("This derive macro only works on struct types.")
    };
    if fields.len() != 1 {
        panic!("This derive macro only supports types with a single field.");
    }
    let field = fields.into_iter().next().unwrap();
    if field.ident.is_some() {
        panic!(
            "Unexpected field {} found while deriving XmlWrapper.",
            field.ident.unwrap()
        );
    }

    let mut result = Vec::new();

    writeln!(
        result,
        "impl From<{wrapper_type}> for crate::xml::XmlElement {{"
    )
    .unwrap();
    writeln!(result, "   fn from(value: {wrapper_type}) -> Self {{").unwrap();
    writeln!(result, "      value.0").unwrap();
    writeln!(result, "   }}").unwrap();
    writeln!(result, "}}").unwrap();

    writeln!(
        result,
        "impl From<crate::xml::XmlElement> for {wrapper_type} {{"
    )
    .unwrap();
    writeln!(
        result,
        "   fn from(value: crate::xml::XmlElement) -> Self {{"
    )
    .unwrap();
    writeln!(result, "      {wrapper_type}(value)").unwrap();
    writeln!(result, "   }}").unwrap();
    writeln!(result, "}}").unwrap();

    writeln!(result, "impl crate::xml::XmlWrapper for {wrapper_type} {{").unwrap();
    writeln!(result, "   fn as_xml(&self) -> &crate::xml::XmlElement {{").unwrap();
    writeln!(result, "      &self.0").unwrap();
    writeln!(result, "   }}").unwrap();
    writeln!(result, "}}").unwrap();

    let result = String::from_utf8(result).unwrap();
    result.parse().unwrap()
}

/// This procedural macro derives the implementation of `XmlProperty` for a specific type
/// and attribute name. It requires that you annotate the struct with
/// `#[property_name("attribute_name")]` and `#[property_type(T)]`. Furthermore, the specified
/// `property_type` must implement `XmlPropertyType`, and the annotated type must have one
/// lifetime parameter, which refers to the lifetime of the underlying XML element.
///
/// Based on https://stackoverflow.com/questions/56188700/how-do-i-make-my-custom-derive-macro-accept-trait-generic-parameters
#[proc_macro_derive(XmlProperty, attributes(property_name, property_type))]
pub fn derive_xml_property(item: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(item).unwrap();
    let wrapper_type = ast.ident.to_string();
    let property_name = read_property_name(&ast);
        let property_type = read_property_type(&ast);

        // Chececk that there is just one field.
    let fields = if let syn::Data::Struct(x) = ast.data {
        x.fields
    } else {
        panic!("This derive macro only works on struct types.")
    };
    if fields.len() != 1 {
        panic!("This derive macro only supports types with a single field.");
    }
    let field = fields.into_iter().next().unwrap();
    if field.ident.is_some() {
        panic!(
            "Unexpected field {} found while deriving XmlWrapper.",
            field.ident.unwrap()
        );
    }

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
                // As opposed to `self.read_raw().is_some()`, this does not need to copy.
                let doc = self.0.read_doc();
                self.0.element()
                    .attribute(std::ops::Deref::deref(&doc), "{property_name}")
                    .is_some()
            }}

            fn is_valid(&self) -> bool {{
                self.read_checked().is_ok()
            }}

            fn read(&self) -> {property_type} {{
                match self.read_checked() {{
                    Ok(result) => result,
                    Err(message) => {{
                        panic!("Cannot read property `{property_name}`: {{message}}")
                    }}
                }}
            }}

            fn read_checked(&self) -> Result<{property_type}, String> {{
                let doc = self.0.read_doc();
                let value = self.0.element()
                    .attribute(std::ops::Deref::deref(&doc), "{property_name}");
                crate::xml::XmlPropertyType::try_read(value)
            }}

            fn read_raw(&self) -> Option<String> {{
                let doc = self.0.read_doc();
                self.0.element()
                    .attribute(std::ops::Deref::deref(&doc), "{property_name}")
                    .map(|it| it.to_string())
            }}

            fn clear(&self) {{
                let mut doc = self.0.write_doc();
                self.0.element()
                    .mut_attributes(std::ops::DerefMut::deref_mut(&mut doc))
                    .remove(&"{property_name}".to_string());
            }}

            fn write(&self, value: &{property_type}) {{
                if let Some(value) = crate::xml::XmlPropertyType::write(value) {{
                    self.write_raw(value);
                }} else {{
                    self.clear();
                }}
            }}

            fn write_raw(&self, value: String) {{
                let mut doc = self.0.write_doc();
                self.0.element()
                    .set_attribute(std::ops::DerefMut::deref_mut(&mut doc), "{property_name}", value);
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
    let result = format!(r#"
        impl crate::sbase::SBase for {ttype} {{}}
    "#);
    result.parse().unwrap()
}

/// This procedural macro derives the implementation of `XmlChild` for a specific type
/// and element name. It requires that you annotate the struct with
/// `#[child_name("attribute_name")]` and `#[child_type(T)]`. Furthermore, the specified
/// `child_type` must implement `XmlWrapper`, and the annotated type must have one
/// lifetime parameter, which refers to the lifetime of the underlying XML element.
///
#[proc_macro_derive(XmlChild, attributes(child_name, child_type))]
pub fn derive_xml_child(item: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(item).unwrap();
    let wrapper_type = ast.ident.to_string();
    let child_name = read_child_name(&ast);
    let child_type = read_child_type(&ast);

    // Chececk that there is just one field.
    let fields = if let syn::Data::Struct(x) = ast.data {
        x.fields
    } else {
        panic!("This derive macro only works on struct types.")
    };
    if fields.len() != 1 {
        panic!("This derive macro only supports types with a single field.");
    }
    let field = fields.into_iter().next().unwrap();
    if field.ident.is_some() {
        panic!(
            "Unexpected field {} found while deriving XmlChild.",
            field.ident.unwrap()
        );
    }

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
                let doc = self.0.read_doc();
                self.0
                    .element()
                    .find(std::ops::Deref::deref(&doc), "{child_name}")
                    .is_some()
            }}

            fn clear(&self) {{
                let mut doc = self.0.write_doc();
                let parent = self.0.element();
                let Some(to_remove) = parent.find(std::ops::Deref::deref(&doc), "{child_name}") else {{
                    return;
                }};
                to_remove
                    .detatch(std::ops::DerefMut::deref_mut(&mut doc))
                    .expect("You can't detach the container element.");
            }}

            fn get(&self) -> {child_type} {{
                self.get_raw()
                    .map(|it| it.into())
                    .unwrap_or_else(|| panic!("Child element `{child_name}` is missing."))
            }}

            fn get_raw(&self) -> Option<crate::xml::XmlElement> {{
                let doc = self.0.read_doc();
                let parent = self.0.element();
                parent
                    .find(std::ops::Deref::deref(&doc), "{child_name}")
                    .map(|it| XmlElement::new(self.0.document(), it))
            }}

            fn set(&self, element: {child_type}) -> Option<{child_type}> {{
                self.set_raw(element.into()).map(|it| it.into())
            }}

            fn set_raw(&self, element: crate::xml::XmlElement) -> Option<crate::xml::XmlElement> {{
                let mut doc = self.0.write_doc();
                let parent = self.0.element();

                // First, remove the existing child.
                let removed = if let Some(to_remove) = parent.find(std::ops::Deref::deref(&doc), "{child_name}") {{
                    to_remove
                        .detatch(std::ops::DerefMut::deref_mut(&mut doc))
                        .expect("You can't detach the container element.");
                    Some(XmlElement::new(self.0.document(), to_remove))
                }} else {{
                    None
                }};

                // Now, push the new child and check that the result is ok.
                let result = parent.push_child(std::ops::DerefMut::deref_mut(&mut doc), element.element().as_node());
                match result {{
                    Err(xml_doc::Error::HasAParent) => {{
                        panic!("Cannot set child. The given element already has a parent.")
                    }}
                    Err(xml_doc::Error::ContainerCannotMove) => {{
                        panic!("Cannot attach the container element to a parent.")
                    }}
                    _ => (),
                }};

                // Return the old child.
                removed
            }}
        }}
        "#
    );

    result.parse().unwrap()
}

fn read_property_name(ast: &syn::DeriveInput) -> String {
    for attr in &ast.attrs {
        if attr.path.is_ident("property_name") {
            let arg: syn::LitStr = attr
                .parse_args()
                .expect("Invalid argument for the `property_name` attribute.");
            return arg.value();
        }
    }
    panic!("Missing `#[property_name(name)]` on a struct annotated with `#[derive(XmlProperty)]`.")
}

fn read_property_type(ast: &syn::DeriveInput) -> String {
    for attr in &ast.attrs {
        if attr.path.is_ident("property_type") {
            let arg: syn::Type = attr
                .parse_args()
                .expect("Invalid argument for the `property_type` attribute.");
            return format!("{}", arg.into_token_stream());
        }
    }
    panic!("Missing `#[property_type(T)]` on a struct annotated with `#[derive(XmlProperty)]`.")
}

fn read_child_name(ast: &syn::DeriveInput) -> String {
    for attr in &ast.attrs {
        if attr.path.is_ident("child_name") {
            let arg: syn::LitStr = attr
                .parse_args()
                .expect("Invalid argument for the `child_name` attribute.");
            return arg.value();
        }
    }
    panic!("Missing `#[child_name(name)]` on a struct annotated with `#[derive(XmlChild)]`.")
}

fn read_child_type(ast: &syn::DeriveInput) -> String {
    for attr in &ast.attrs {
        if attr.path.is_ident("child_type") {
            let arg: syn::Type = attr
                .parse_args()
                .expect("Invalid argument for the `child_type` attribute.");
            return format!("{}", arg.into_token_stream());
        }
    }
    panic!("Missing `#[child_type(T)]` on a struct annotated with `#[derive(XmlChild)]`.")
}