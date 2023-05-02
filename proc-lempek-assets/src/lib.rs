use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput};

#[proc_macro_derive(SerJsonBody)]
pub fn serialize_to_json_body(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let output = quote! {
        impl actix_web::Responder for #name {
            type Body = actix_web::body::BoxBody;

            fn respond_to(self, _req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
                let body = serde_json::to_string(&self).unwrap();

                actix_web::HttpResponse::Ok()
                    .content_type(actix_web::http::header::ContentType::json())
                    .body(body)
            }
        }
    };
    output.into()
}