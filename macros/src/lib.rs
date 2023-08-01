extern crate proc_macro;
use proc_macro::TokenStream;
use std::io::Write;

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
