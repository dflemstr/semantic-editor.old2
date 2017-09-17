extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate semantic;
extern crate syn;

#[proc_macro_derive(Semantic)]
pub fn semantic(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();
    let gen = impl_semantic(&ast);
    gen.into()
}

fn impl_semantic(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    quote! {
        extern crate semantic as _semantic;
        impl _semantic::Semantic for #name {
        }
    }
}
