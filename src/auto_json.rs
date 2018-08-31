#![feature(proc_macro, proc_macro_lib)]
#![crate_type = "proc-macro"]

extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
extern crate quote;

use proc_macro::TokenStream;

#[proc_macro_derive(IntoJSON)]
pub fn into_json(input: TokenStream) -> TokenStream {
	let s = input.to_string();
	let ast = syn::parse_derive_input(&s).unwrap();
	let gen = impl_into_json(&ast);

	gen.parse().unwrap()
}

fn impl_into_json(ast: &syn::DeriveInput) -> quote::Tokens {
	let name = &ast.ident;
	if let syn::Body::Struct(_) = ast.body {
		quote! {
			impl IntoJSON for #name {
				fn into_json(&self, builder: &mut JSONBuilder) -> Result {
					// TODO
				}
			}
		}
	} else {
		panic!("#[derive(IntoJSON)] is only defined for structs, not for enums!");
	}
}