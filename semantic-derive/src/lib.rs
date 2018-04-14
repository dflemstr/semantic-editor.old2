#![recursion_limit = "128"]

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate semantic;
extern crate syn;

#[derive(Debug, Default)]
struct Attributes {
    kind: Option<semantic::Kind>,
    role: Option<semantic::Role>,
}

#[derive(Debug, Default)]
struct FieldAttributes {
    name: String,
    cardinality: Option<semantic::Cardinality>,
    is_children: bool,
}

#[proc_macro_derive(Semantic, attributes(semantic))]
pub fn semantic(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();
    let gen = impl_semantic(&ast);
    gen.into()
}

fn impl_semantic(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    let name_str = name.as_ref();
    let attributes = Attributes::from(ast.attrs.as_slice());
    let field_attributes = field_attributes(ast);

    let kind = syn::Ident::from(format!(
        "{:?}",
        attributes
            .kind
            .or_else(|| infer_kind(ast))
            .expect("can't determine kind; annotate explicitly with #[semantic(kind = \"...\")]")
    ));

    let role = syn::Ident::from(format!(
        "{:?}",
        attributes
            .role
            .expect("missing role attribute, like #[semantic(role = \"...\")]")
    ));

    let field_names = field_attributes.iter().map(|f| f.name.as_str()).collect::<Vec<_>>();
    let field_is_childrens = field_attributes.iter().map(|f| f.is_children).collect::<Vec<_>>();

    quote! {
        impl ::semantic::Semantic for #name {
            const CLASS: ::semantic::Class<'static> = ::semantic::Class {
                name: #name_str,
                id: ::std::any::TypeId::of::<#name>(),
                kind: ::semantic::Kind::#kind,
                role: ::semantic::Role::#role,
                fields: &[#(
                    ::semantic::Field {
                        name: #field_names,
                        is_children: #field_is_childrens,
                    },
                )*],
            };

            fn field(&self, _field: &str) {}

            fn field_mut(&mut self, _field: &str) {}

            fn variant(&self, _variant: &str) {}

            fn variant_mut(&mut self, _variant: &str) {}
        }
    }
}

fn field_attributes(ast: &syn::DeriveInput) -> Vec<FieldAttributes> {
    match ast.data {
        syn::Data::Struct(syn::DataStruct { ref fields, .. }) => struct_field_attributes(fields),
        syn::Data::Enum(syn::DataEnum { ref variants, .. }) => {
            enum_field_attributes(variants.iter().collect::<Vec<_>>().as_slice())
        }
        syn::Data::Union(_) => panic!("Deriving not supported for unions"),
    }
}

fn struct_field_attributes(fields: &syn::Fields) -> Vec<FieldAttributes> {
    build_field_attributes(fields.iter())
}

fn enum_field_attributes(variants: &[&syn::Variant]) -> Vec<FieldAttributes> {
    // TODO
    vec![]
}

fn build_field_attributes<'a, I>(fields: I) -> Vec<FieldAttributes>
where
    I: Iterator<Item = &'a syn::Field>,
{
    fields
        .map(|f| {
            let mut result = FieldAttributes::from(f.attrs.as_slice());
            result.name = f.ident
                .expect("unnamed fields not supported for #[semantic(kind = \"record\")]")
                .as_ref()
                .to_owned();
            result
        })
        .collect()
}

impl<'a> From<&'a [syn::Attribute]> for Attributes {
    fn from(attrs: &'a [syn::Attribute]) -> Self {
        use syn::NestedMeta::*;

        let mut result = Attributes::default();

        for meta_items in attrs.iter().filter_map(get_semantic_meta_items) {
            for meta_item in meta_items {
                match meta_item {
                    Meta(syn::Meta::NameValue(ref m)) if m.ident == "kind" => {
                        if let Ok(s) = get_lit_str(m.ident.as_ref(), m.ident.as_ref(), &m.lit) {
                            let value = s.value();
                            match value.parse() {
                                Ok(kind) => result.kind = Some(kind),
                                Err(()) => panic!("invalid kind value {:?}", value),
                            }
                        }
                    }
                    Meta(syn::Meta::NameValue(ref m)) if m.ident == "role" => {
                        if let Ok(s) = get_lit_str(m.ident.as_ref(), m.ident.as_ref(), &m.lit) {
                            let value = s.value();
                            match value.parse() {
                                Ok(role) => result.role = Some(role),
                                Err(()) => panic!("invalid role value {:?}", value),
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        result
    }
}

impl<'a> From<&'a [syn::Attribute]> for FieldAttributes {
    fn from(attrs: &'a [syn::Attribute]) -> Self {
        use syn::NestedMeta::*;

        let mut result = FieldAttributes::default();

        for meta_items in attrs.iter().filter_map(get_semantic_meta_items) {
            for meta_item in meta_items {
                match meta_item {
                    Meta(syn::Meta::Word(ref ident)) if ident == "children" => {
                        result.is_children = true
                    }
                    _ => {}
                }
            }
        }

        result
    }
}

fn infer_kind(ast: &syn::DeriveInput) -> Option<semantic::Kind> {
    match ast.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Unit,
            ..
        }) => Some(semantic::Kind::Unit),
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(_),
            ..
        }) => Some(semantic::Kind::Record),
        syn::Data::Enum(syn::DataEnum { ref variants, .. }) => {
            if variants.iter().all(is_union_variant) {
                Some(semantic::Kind::Union)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn is_union_variant(variant: &syn::Variant) -> bool {
    match variant.fields {
        syn::Fields::Unnamed(syn::FieldsUnnamed { ref unnamed, .. }) => unnamed.len() == 1,
        _ => false,
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
