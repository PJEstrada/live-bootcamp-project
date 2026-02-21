use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn test_with_cleanup(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let fn_stmts = &input.block.stmts;

    quote! {
        #[tokio::test]
        async fn #fn_name() {
            let mut app = TestApp::new().await;
            #(#fn_stmts)*
            app.clean_up().await;
        }
    }
    .into()
}