#![recursion_limit = "128"]

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate semantic;
extern crate syn;

struct Attributes {
    role: Option<semantic::Role>,
}

struct FieldAttributes {
    ident: syn::Ident,
    ty: syn::Type,
    is_children: bool,
}

struct VariantAttributes {
    ident: syn::Ident,
    ty: syn::Type,
}

#[proc_macro_derive(Semantic, attributes(semantic))]
pub fn semantic(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();
    let gen = impl_semantic(ast);
    gen.into()
}

fn impl_semantic(ast: syn::DeriveInput) -> proc_macro2::TokenStream {
    let name = &ast.ident;
    let mut attributes = Attributes::new();
    attributes.set_from_attrs(ast.attrs.as_slice());

    let InferredStructure {
        structure,
        visit_classes,
    } = infer_structure(&ast).expect("can't infer semantic structure; is your type too complex?");

    let visit_classes = visit_classes
        .map(|v| {
            quote! {
                fn visit_classes<F>(visitor: &mut F) where F: FnMut(&'static ::semantic::Class<'static>) -> bool {
                    #v
                }
            }
        })
        .unwrap_or(quote!());

    let role = syn::Ident::new(
        &format!(
            "{:?}",
            attributes
                .role
                .expect("missing role attribute, like #[semantic(role = \"...\")]")
        ),
        ast.ident.span(),
    );

    quote! {
        impl ::semantic::Semantic for #name {
            const CLASS: ::semantic::Class<'static> = ::semantic::Class {
                id: <Self as ::type_info::TypeInfo>::TYPE.id,
                role: ::semantic::Role::#role,
                structure: #structure,
            };

            #visit_classes
        }

        impl ::semantic::DynamicSemantic for #name {
            fn class(&self) -> ::semantic::Class<'static> {
                <Self as ::semantic::Semantic>::CLASS
            }
        }
    }
}

struct InferredStructure {
    structure: proc_macro2::TokenStream,
    visit_classes: Option<proc_macro2::TokenStream>,
}

fn infer_structure(ast: &syn::DeriveInput) -> Option<InferredStructure> {
    let name = ast.ident.to_string();
    match ast.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Unit,
            ..
        }) => Some(InferredStructure {
            structure: quote!(::semantic::Structure::Unit {
                name: concat!(module_path!(), "::", #name),
            }),
            visit_classes: None,
        }),
        syn::Data::Struct(syn::DataStruct { ref fields, .. }) => {
            let field_attributes = struct_field_attributes(fields);
            let field_names = field_attributes.iter().map(|f| f.ident.to_string());
            let field_types1 = field_attributes.iter().map(|f| &f.ty);
            let field_types2 = field_attributes.iter().map(|f| &f.ty);
            let field_types3 = field_attributes.iter().map(|f| &f.ty);
            let field_is_childrens = field_attributes.iter().map(|f| f.is_children);

            Some(InferredStructure {
                structure: quote! {
                    ::semantic::Structure::Record {
                        name: concat!(module_path!(), "::", #name),
                        fields: &[#(
                            ::semantic::Field {
                                name: #field_names,
                                ty: ::std::any::TypeId::of::<#field_types1>(),
                                is_children: #field_is_childrens,
                            },
                        )*]
                    }
                },
                visit_classes: Some(quote! {
                    #(
                        if visitor(&<#field_types2 as ::semantic::Semantic>::CLASS) {
                            <#field_types3 as ::semantic::Semantic>::visit_classes(visitor);
                        }
                    )*
                }),
            })
        }
        syn::Data::Enum(syn::DataEnum { ref variants, .. }) => {
            if variants.iter().all(is_union_variant) {
                let variant_attributes = enum_union_variant_attributes(variants.iter());
                let variant_names = variant_attributes.iter().map(|f| f.ident.to_string());
                let variant_types1 = variant_attributes.iter().map(|f| &f.ty);
                let variant_types2 = variant_attributes.iter().map(|f| &f.ty);
                let variant_types3 = variant_attributes.iter().map(|f| &f.ty);

                Some(InferredStructure {
                    structure: quote! {
                        ::semantic::Structure::Union {
                            variants: &[#(
                                ::semantic::Variant {
                                    name: #variant_names,
                                    ty: ::std::any::TypeId::of::<#variant_types1>(),
                                },
                            )*],
                        }
                    },
                    visit_classes: Some(quote! {
                        #(
                            if visitor(&#variant_types2::CLASS) {
                                #variant_types3::visit_classes(visitor);
                            }
                        )*
                    }),
                })
            } else if variants.iter().all(is_enumeration_variant) {
                Some(InferredStructure {
                    structure: quote! {
                        ::semantic::Structure::Enumeration {
                            variants: &[],
                        }
                    },
                    visit_classes: None,
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

fn struct_field_attributes(fields: &syn::Fields) -> Vec<FieldAttributes> {
    build_field_attributes(fields.iter())
}

fn build_field_attributes<'a, I>(fields: I) -> Vec<FieldAttributes>
where
    I: Iterator<Item = &'a syn::Field>,
{
    fields
        .filter(|f| f.ident.is_some())
        .map(|f| {
            let mut result = FieldAttributes::new(f.ident.as_ref().unwrap().clone(), f.ty.clone());
            result.set_from_attrs(f.attrs.as_slice());
            result
        }).collect()
}

fn enum_union_variant_attributes<'a, I>(variants: I) -> Vec<VariantAttributes>
where
    I: Iterator<Item = &'a syn::Variant>,
{
    variants
        .map(|v| VariantAttributes {
            ident: v.ident.clone(),
            ty: match v.fields {
                syn::Fields::Unnamed(syn::FieldsUnnamed { ref unnamed, .. }) => {
                    // We have already established that this is an union enum, so this should be
                    // safe
                    unnamed.iter().next().unwrap().ty.clone()
                }
                _ => unreachable!(),
            },
        }).collect()
}

impl Attributes {
    fn new() -> Attributes {
        Attributes { role: None }
    }

    fn set_from_attrs(&mut self, attrs: &[syn::Attribute]) {
        use syn::NestedMeta::*;

        for meta_items in attrs.iter().filter_map(get_semantic_meta_items) {
            for meta_item in meta_items {
                match meta_item {
                    Meta(syn::Meta::NameValue(ref m)) if m.ident == "role" => {
                        if let Ok(s) =
                            get_lit_str(&m.ident.to_string(), &m.ident.to_string(), &m.lit)
                        {
                            let value = s.value();
                            match value.parse() {
                                Ok(role) => self.role = Some(role),
                                Err(()) => panic!("invalid role value {:?}", value),
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

impl FieldAttributes {
    fn new(ident: syn::Ident, ty: syn::Type) -> FieldAttributes {
        let is_children = false;
        FieldAttributes {
            ident,
            ty,
            is_children,
        }
    }

    fn set_from_attrs(&mut self, attrs: &[syn::Attribute]) {
        use syn::NestedMeta::*;

        for meta_items in attrs.iter().filter_map(get_semantic_meta_items) {
            for meta_item in meta_items {
                match meta_item {
                    Meta(syn::Meta::Word(ref ident)) if ident == "children" => {
                        self.is_children = true
                    }
                    _ => {}
                }
            }
        }
    }
}

fn is_enumeration_variant(variant: &syn::Variant) -> bool {
    match variant.fields {
        syn::Fields::Unit => true,
        _ => false,
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
