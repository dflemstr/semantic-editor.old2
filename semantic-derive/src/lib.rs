extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate semantic;
extern crate syn;

#[derive(Debug, Default)]
struct Attributes {
    kind: Option<semantic::Kind>,
}

#[derive(Debug, Default)]
struct FieldAttributes {
    is_children: bool,
    cardinality: Option<semantic::Cardinality>,
}

#[proc_macro_derive(Semantic, attributes(semantic))]
pub fn semantic(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();
    let gen = impl_semantic(&ast);
    gen.into()
}

fn impl_semantic(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    let type_ = name.as_ref();
    let attributes = Attributes::from(ast);

    let kind = syn::Ident::from(format!(
        "{:?}",
        attributes
            .kind
            .expect("missing kind attribute, like #[semantic(kind = \"...\")]")
    ));

    quote! {
        extern crate semantic as _semantic;
        impl _semantic::Semantic for #name {
            fn type_() -> &'static str {
                #type_
            }

            fn kind() -> _semantic::Kind {
                _semantic::Kind::#kind
            }
        }
    }
}

impl<'a> From<&'a syn::DeriveInput> for Attributes {
    fn from(ast: &syn::DeriveInput) -> Self {
        use syn::NestedMeta::*;

        let mut attributes = Attributes::default();

        for meta_items in ast.attrs.iter().filter_map(get_semantic_meta_items) {
            for meta_item in meta_items {
                match meta_item {
                    Meta(syn::Meta::NameValue(ref m)) if m.ident == "kind" => {
                        if let Ok(s) = get_lit_str(m.ident.as_ref(), m.ident.as_ref(), &m.lit) {
                            let value = s.value();
                            match value.parse() {
                                Ok(kind) => attributes.kind = Some(kind),
                                Err(()) => panic!("invalid kind value {:?}", value),
                            }
                        }
                    }
                    Meta(syn::Meta::Word(ref ident)) if ident == "children" => { /* TODO */ }
                    _ => {}
                }
            }
        }

        attributes
    }
}

fn get_semantic_meta_items(attr: &syn::Attribute) -> Option<Vec<syn::NestedMeta>> {
    if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "semantic" {
        match attr.interpret_meta() {
            Some(syn::Meta::List(ref meta)) => Some(meta.nested.iter().cloned().collect()),
            _ => None,
        }
    } else {
        None
    }
}

fn get_lit_str<'a>(
    attr_name: &str,
    meta_item_name: &str,
    lit: &'a syn::Lit,
) -> Result<&'a syn::LitStr, ()> {
    if let syn::Lit::Str(ref lit) = *lit {
        Ok(lit)
    } else {
        format!(
            "expected semantic {} attribute to be a string: `{} = \"...\"`",
            attr_name, meta_item_name
        );
        Err(())
    }
}
