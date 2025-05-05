extern crate proc_macro;
use proc_macro::TokenStream;

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

        impl crate::xml::XmlWrapper for {wrapper_type} {{
            fn xml_element(&self) -> &crate::xml::XmlElement {{
                &self.0
            }}

            unsafe fn unchecked_cast<T: crate::xml::XmlWrapper>(element: T) -> Self {{
                Self(element.xml_element().clone())
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
        impl crate::core::SBase for {ttype} {{}}
    "#
    );
    result.parse().unwrap()
}

/// Adds a "default" implementation for PythonPropertyType, assuming the type implements
/// all necessary Python conversion traits.
#[proc_macro_derive(PythonPropertyType)]
pub fn derive_python_property(item: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(item).unwrap();
    let ttype = ast.ident.to_string();
    let result = generate_python_property(ttype);
    result.parse().unwrap()
}

#[proc_macro]
pub fn make_python_property(item: TokenStream) -> TokenStream {
    let ttype = item.to_string();
    let result = generate_python_property(ttype);
    result.parse().unwrap()
}

fn generate_python_property(ttype: String) -> String {
    format!(
        r#"
        impl crate::xml::py::PythonPropertyType for {ttype} {{
            fn converter() -> Box<dyn crate::xml::py::PythonPropertyConverter + Send + Sync> {{
                struct Internal;
                impl crate::xml::py::PythonPropertyConverter for Internal {{
                    fn try_from_string(&self, value: String, py: pyo3::Python) -> pyo3::PyResult<pyo3::PyObject> {{
                        use crate::xml::XmlPropertyType;
                        use pyo3::conversion::IntoPyObjectExt;
                        let input = Some(value.as_str());
                        let value: Option<{ttype}> = {ttype}::try_get(input)
                            .map_err(crate::xml::py::runtime_error)?;
                        match value {{
                            Some(value) => value.into_py_any(py),
                            None => Ok(py.None()),
                        }}
                    }}

                    fn try_into_string(&self, value: pyo3::PyObject, py: pyo3::Python) -> pyo3::PyResult<String> {{
                        use crate::xml::XmlPropertyType;
                        let value = value.extract::<{ttype}>(py)?;
                        Ok(value.set().unwrap())
                    }}
                }}
                Box::new(Internal {{}})
            }}
        }}

    "#
    )
}
