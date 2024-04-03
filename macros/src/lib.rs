use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Fields};

#[proc_macro_derive(Vertex)]
pub fn derive_vertex(item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as syn::ItemStruct);
    let ident = item.ident;
    let fields = item.fields;
    let (_field_types, names) = if let Fields::Named(fields) = fields {
        let types = fields
            .named
            .iter()
            .map(|field| field.ty.clone())
            .collect::<Vec<_>>();
        let names = fields
            .named
            .iter()
            .map(|field| field.ident.clone())
            .collect::<Vec<_>>();
        (types, names)
    } else {
        panic!("Only structs with named fields are supported");
    };

    quote! {
        #[automatically_derived]
        impl Into<glium2::buffer::VertexData> for #ident {
            fn into(self) -> glium2::buffer::VertexData {
                let mut data = vec![];
                #(data.extend(std::simd::ToBytes::to_ne_bytes(self.#names).iter().collect::<Vec<_>>());)*

                glium2::buffer::VertexData {
                    data
                }
            }
        }

        #[automatically_derived]
        impl glium2::shader::Vertex for #ident {
            fn get_vertex_spec() -> glium2::shader::VertexAttributeSpec {

            }
        }
    }
    .into()
}
