extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{bracketed, LitStr, Token};

/// The absolute path of the library crate these macros generate code for.
///
/// The generated code has to be usable from a type declared *outside* the library — an SBML
/// extension is exactly such a type, and the whole point of the macros is that an extension is
/// declared the same way the core objects are. A `crate::…` path would silently mean "the crate
/// that invokes the macro", so every path is absolute instead; the library re-exports itself under
/// its own name for that purpose.
fn library() -> proc_macro2::TokenStream {
    quote! { ::biodivine_lib_sbml }
}

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

/// The parsed contents of a `#[sbml_object(..)]` attribute.
///
/// The attribute cannot be parsed with `syn::meta`, because a value that is not a literal — the
/// conventional `namespace = NS_SBML_CORE` — is not a `syn::MetaNameValue`, so the arguments are
/// read with a small hand-written parser instead.
#[derive(Default)]
struct SbmlObjectArgs {
    /// The `name = "…"` argument.
    name: Option<LitStr>,
    /// The `names = ["…", …]` argument.
    names: Option<Vec<LitStr>>,
    /// The `namespace = <expression>` argument.
    namespace: Option<syn::Expr>,
    /// The `xsi_type = "…"` argument.
    xsi_type: Option<LitStr>,
    /// The `constructor = <bool>` argument; `true` (the default) means that the declaration
    /// generates an inherent `new_empty(document)`.
    constructor: Option<syn::LitBool>,
}

impl Parse for SbmlObjectArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = SbmlObjectArgs::default();
        while !input.is_empty() {
            let key: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "name" => args.name = Some(input.parse()?),
                "names" => {
                    let content;
                    bracketed!(content in input);
                    // `LitStr::parse` is the inherent method of `LitStr`, which returns the
                    // literal itself and shadows the `Parse` implementation; the trait method is
                    // named explicitly here.
                    let names: Punctuated<LitStr, Token![,]> =
                        content.parse_terminated(<LitStr as Parse>::parse)?;
                    args.names = Some(names.into_iter().collect());
                }
                "namespace" => args.namespace = Some(input.parse()?),
                "xsi_type" => args.xsi_type = Some(input.parse()?),
                "constructor" => args.constructor = Some(input.parse()?),
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unknown `sbml_object` argument `{other}`; expected `name`, `names`, \
                             `namespace`, `xsi_type` or `constructor`"
                        ),
                    ))
                }
            }
            if input.is_empty() {
                break;
            }
            input.parse::<Token![,]>()?;
        }
        Ok(args)
    }
}

/// Reads the `#[sbml_object(..)]` attribute of the annotated item, if there is one (and complains
/// if there are several, because two identities for one type cannot be resolved).
fn read_sbml_object_attribute(ast: &syn::DeriveInput) -> syn::Result<Option<SbmlObjectArgs>> {
    let mut found: Option<SbmlObjectArgs> = None;
    for attribute in ast.attrs.iter() {
        if !attribute.path.is_ident("sbml_object") {
            continue;
        }
        if found.is_some() {
            return Err(syn::Error::new_spanned(
                attribute,
                "a type can only carry one `#[sbml_object(..)]` attribute",
            ));
        }
        found = Some(attribute.parse_args::<SbmlObjectArgs>()?);
    }
    Ok(found)
}

/// Builds the `SbmlObject` implementation for a type from its parsed attribute, together with (unless
/// the declaration opts out with `constructor = false`) its inherent identity-derived constructor.
fn sbml_object_impl(
    wrapper_type: &syn::Ident,
    args: SbmlObjectArgs,
) -> syn::Result<proc_macro2::TokenStream> {
    let library = library();
    let namespace = match args.namespace {
        Some(namespace) => namespace,
        None => {
            return Err(syn::Error::new_spanned(
                wrapper_type,
                "`#[sbml_object(..)]` requires the namespace of the element, for example \
                 `namespace = NS_SBML_CORE`",
            ))
        }
    };
    let names: Vec<LitStr> =
        match (args.name, args.names) {
            (Some(name), None) => vec![name],
            (None, Some(names)) if !names.is_empty() => names,
            (None, Some(_)) => {
                return Err(syn::Error::new_spanned(
                    wrapper_type,
                    "`#[sbml_object(names = [..])]` requires at least one element name",
                ))
            }
            (Some(_), Some(_)) => return Err(syn::Error::new_spanned(
                wrapper_type,
                "`#[sbml_object(..)]` accepts either `name = \"…\"` or `names = [..]`, not both",
            )),
            (None, None) => {
                return Err(syn::Error::new_spanned(
                    wrapper_type,
                    "`#[sbml_object(..)]` requires `name = \"…\"` or `names = [\"…\", …]`",
                ))
            }
        };
    let primary = &names[0];
    // A type whose element has several names (a supertype such as `GraphicalObject`) writes its
    // `ELEMENT_NAME` as the first of them and overrides `element_names` with the whole list, so a
    // name appears exactly once in the declaration.
    let element_names = if names.len() > 1 {
        quote! {
            fn element_names() -> &'static [&'static str] {
                &[#(#names),*]
            }
        }
    } else {
        quote! {}
    };
    let xsi_type = args.xsi_type;
    let xsi_type_const = match &xsi_type {
        Some(xsi_type) => quote! { const XSI_TYPE: Option<&'static str> = Some(#xsi_type); },
        None => quote! {},
    };

    // The identity-derived constructor. A declaration opts out with `constructor = false` when
    // `names[0]` is the element name of a concrete sibling instead of an element of its own, so
    // that no caller can accidentally create the sibling's element through this type; such a type
    // is constructed through its named form (`SbmlUtils::new_empty_named`) or through a subtype.
    let generate_constructor = args.constructor.map(|it| it.value()).unwrap_or(true);
    let (constructor, construct_const) = if generate_constructor {
        // A type discriminated by `xsi:type` (the cubic bezier variant of a layout curve segment)
        // needs the attribute on the new element, or it would not be recognized as itself:
        // `xsi:type` lives in the XML Schema instance namespace, which a fresh element does not
        // bind, so the declaration is written on the element as well.
        let set_xsi_type = match &xsi_type {
            Some(xsi_type) => quote! { #library::xml::set_xsi_type(&element, #xsi_type); },
            None => quote! {},
        };
        let constructor = quote! {
            impl #wrapper_type {
                /// Create a new, empty element of this type in the given document.
                ///
                /// The element is built from the declared identity of the type — its element name,
                /// its namespace and, for a type that is discriminated that way, its `xsi:type`
                /// attribute — and it is detached (it has no parent).
                ///
                /// The element is **not** guaranteed to be valid: the required attributes and
                /// children of the type are not filled in, and an identifier is not assigned. Use
                /// the typed constructor of the type (for example `Species::new`) to build a
                /// complete object, and [`SbmlObject`](::biodivine_lib_sbml::xml::SbmlObject) to
                /// check that an existing element really is an instance of the type.
                pub fn new_empty(document: #library::xml::Document) -> Self {
                    let element = #library::xml::XmlElement::new_qualified(
                        document,
                        <Self as #library::xml::SbmlObject>::ELEMENT_NAME,
                        <Self as #library::xml::SbmlObject>::ELEMENT_NAMESPACE,
                    );
                    #set_xsi_type
                    Self::from(element)
                }
            }
        };
        let construct_const = quote! {
            const CONSTRUCT: Option<fn(#library::xml::Document) -> Self> = Some(Self::new_empty);
        };
        (constructor, construct_const)
    } else {
        (quote! {}, quote! {})
    };

    Ok(quote! {
        impl #library::xml::SbmlObject for #wrapper_type {
            const ELEMENT_NAME: &'static str = #primary;
            const ELEMENT_NAMESPACE: #library::xml::Namespace = #namespace;
            #element_names
            #xsi_type_const
            #construct_const
        }
        #constructor
    })
}

/// This procedural macro takes the declaration of a tuple `struct` type and derives the
/// following trait implementations for it:
///  - `XmlWrapper for T` (the trait every SBML object implements)
///  - `From<XmlElement> for T` and `From<T> for XmlElement` (the unchecked conversions)
///  - `SbmlObject for T`, when the declaration carries a `#[sbml_object(..)]` attribute
///
/// The only requirement is that the type must have a single anonymous field whose type is
/// `XmlElement`.
///
/// The optional `#[sbml_object(name = "…", namespace = …)]` attribute declares which element of
/// which namespace the type stands for; see the documentation of `SbmlObject` for what the
/// declaration is used for. A type whose element has several names (a supertype) writes
/// `names = ["…", "…"]` instead of `name`, a type whose element is told apart from its sibling by
/// `xsi:type` rather than by its local name adds `xsi_type = "…"`, and a supertype whose first name
/// belongs to a *concrete* sibling writes `constructor = false` to suppress the identity-derived
/// `new_empty` constructor (the named form and the subtypes stay available).
///
/// The attribute also generates an inherent `new_empty(document)` constructor for every type that
/// does not opt out, so that an object is always created as the declaration says it should be.
#[proc_macro_derive(XmlWrapper, attributes(sbml_object))]
pub fn derive_xml_wrapper(item: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(item).unwrap();
    let wrapper_type = &ast.ident;
    check_single_field(&ast);

    let library = library();
    let mut result = quote! {
        impl From<#wrapper_type> for #library::xml::XmlElement {
            fn from(value: #wrapper_type) -> Self {
                value.0
            }
        }

        impl From<#library::xml::XmlElement> for #wrapper_type {
            fn from(value: #library::xml::XmlElement) -> Self {
                Self(value)
            }
        }

        impl #library::xml::XmlWrapper for #wrapper_type {
            fn xml_element(&self) -> &#library::xml::XmlElement {
                &self.0
            }
        }
    };

    match read_sbml_object_attribute(&ast) {
        Ok(Some(args)) => match sbml_object_impl(wrapper_type, args) {
            Ok(identity) => result.extend(identity),
            Err(error) => result.extend(error.to_compile_error()),
        },
        Ok(None) => {}
        Err(error) => result.extend(error.to_compile_error()),
    }

    result.into()
}

/// Adds a "default" implementation for SBase which simply inherits all methods.
#[proc_macro_derive(SBase)]
pub fn derive_sbase(item: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(item).unwrap();
    let ttype = &ast.ident;
    let library = library();
    let result = quote! {
        impl #library::core::SBase for #ttype {}
    };
    result.into()
}
