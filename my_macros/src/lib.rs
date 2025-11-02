use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn log_duration(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let sig = &input.sig;
    let name = &sig.ident;
    let block = &input.block;
    let attrs = &input.attrs;
    let vis = &input.vis;

    let result = if sig.asyncness.is_some() {
        quote! {
            #(#attrs)*
            #vis #sig {
                let start = std::time::Instant::now();
                let result = async move #block.await;
                eprintln!("{} took {:?}", stringify!(#name), start.elapsed());
                result
            }
        }
    } else {
        quote! {
            #(#attrs)*
            #vis #sig {
                let start = std::time::Instant::now();
                #block
                eprintln!("{} took {:?}", stringify!(#name), start.elapsed());
            }
        }
    };

    result.into()
}