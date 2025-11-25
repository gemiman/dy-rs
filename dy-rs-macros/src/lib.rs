//! Procedural macros for dy-rs.
//!
//! Currently exposes:
//! - `#[dy_api(...)]` to document handlers and auto-register them for OpenAPI generation.

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Expr, Ident, Lit, LitInt, LitStr, Meta, Token, Type, TypePath, parse_macro_input,
    punctuated::Punctuated, spanned::Spanned,
};

#[derive(Default)]
struct ApiArgs {
    method: Option<Ident>,
    path: Option<LitStr>,
    request: Option<Type>,
    response: Option<Type>,
    status: Option<LitInt>,
    tag: Option<LitStr>,
    summary: Option<LitStr>,
    description: Option<LitStr>,
}

fn parse_args(args: Punctuated<Meta, Token![,]>) -> syn::Result<ApiArgs> {
    let mut out = ApiArgs::default();

    for arg in args {
        match arg {
            Meta::NameValue(nv) if nv.path.is_ident("method") => match nv.value {
                Expr::Path(expr_path) => {
                    if let Some(ident) = expr_path.path.get_ident() {
                        out.method = Some(ident.clone());
                    } else {
                        return Err(syn::Error::new(
                            expr_path.span(),
                            "method must be an identifier (get, post, put, delete, patch)",
                        ));
                    }
                }
                Expr::Lit(expr_lit) => {
                    if let Lit::Str(ref s) = expr_lit.lit {
                        out.method = Some(Ident::new(&s.value(), expr_lit.span()));
                    } else {
                        return Err(syn::Error::new(
                            expr_lit.span(),
                            "method must be an identifier or string",
                        ));
                    }
                }
                other => {
                    return Err(syn::Error::new(
                        other.span(),
                        "method must be an identifier (get, post, put, delete, patch)",
                    ));
                }
            },
            Meta::NameValue(nv) if nv.path.is_ident("path") => {
                if let Expr::Lit(expr_lit) = nv.value {
                    if let Lit::Str(s) = expr_lit.lit {
                        out.path = Some(s);
                    } else {
                        return Err(syn::Error::new(
                            expr_lit.span(),
                            "path must be a string literal",
                        ));
                    }
                }
            }
            Meta::NameValue(nv) if nv.path.is_ident("request") => {
                if let Expr::Path(expr_path) = nv.value {
                    out.request = Some(Type::Path(TypePath {
                        qself: expr_path.qself,
                        path: expr_path.path,
                    }));
                } else {
                    return Err(syn::Error::new(nv.value.span(), "request must be a type"));
                }
            }
            Meta::NameValue(nv) if nv.path.is_ident("response") => {
                if let Expr::Path(expr_path) = nv.value {
                    out.response = Some(Type::Path(TypePath {
                        qself: expr_path.qself,
                        path: expr_path.path,
                    }));
                } else {
                    return Err(syn::Error::new(nv.value.span(), "response must be a type"));
                }
            }
            Meta::NameValue(nv) if nv.path.is_ident("status") => {
                if let Expr::Lit(expr_lit) = nv.value {
                    if let Lit::Int(lit) = expr_lit.lit {
                        out.status = Some(lit);
                    } else {
                        return Err(syn::Error::new(
                            expr_lit.span(),
                            "status must be an integer literal",
                        ));
                    }
                }
            }
            Meta::NameValue(nv) if nv.path.is_ident("tag") => {
                if let Expr::Lit(expr_lit) = nv.value {
                    if let Lit::Str(lit) = expr_lit.lit {
                        out.tag = Some(lit);
                    } else {
                        return Err(syn::Error::new(
                            expr_lit.span(),
                            "tag must be a string literal",
                        ));
                    }
                }
            }
            Meta::NameValue(nv) if nv.path.is_ident("summary") => {
                if let Expr::Lit(expr_lit) = nv.value {
                    if let Lit::Str(lit) = expr_lit.lit {
                        out.summary = Some(lit);
                    } else {
                        return Err(syn::Error::new(
                            expr_lit.span(),
                            "summary must be a string literal",
                        ));
                    }
                }
            }
            Meta::NameValue(nv) if nv.path.is_ident("description") => {
                if let Expr::Lit(expr_lit) = nv.value {
                    if let Lit::Str(lit) = expr_lit.lit {
                        out.description = Some(lit);
                    } else {
                        return Err(syn::Error::new(
                            expr_lit.span(),
                            "description must be a string literal",
                        ));
                    }
                }
            }
            other => {
                return Err(syn::Error::new(
                    other.span(),
                    "unsupported attribute, expected method, path, request, response, status, tag, summary, or description",
                ));
            }
        }
    }

    Ok(out)
}

/// Document a handler for automatic OpenAPI generation.
///
/// Example:
/// ```rust
/// #[dy_api(
///     method = get,
///     path = "/users/{id}",
///     response = User,
///     request = UpdateUserRequest,
///     tag = "Users",
///     summary = "Update a user"
/// )]
/// async fn update_user(...) { ... }
/// ```
#[proc_macro_attribute]
pub fn dy_api(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr with Punctuated<Meta, Token![,]>::parse_terminated);
    let parsed = match parse_args(args) {
        Ok(p) => p,
        Err(err) => return err.to_compile_error().into(),
    };

    let method = parsed
        .method
        .unwrap_or_else(|| Ident::new("get", proc_macro2::Span::call_site()));
    let path = parsed
        .path
        .unwrap_or_else(|| LitStr::new("/", proc_macro2::Span::call_site()));
    let status = parsed
        .status
        .unwrap_or_else(|| LitInt::new("200", proc_macro2::Span::call_site()));
    let status_str = LitStr::new(&status.base10_digits(), status.span());

    let request_ty = parsed.request;
    let response_ty = parsed.response;
    let tag = parsed.tag;
    let summary = parsed.summary;
    let description = parsed.description;

    let method_expr = match method.to_string().as_str() {
        "get" | "GET" => quote! { utoipa::openapi::path::HttpMethod::Get },
        "post" | "POST" => quote! { utoipa::openapi::path::HttpMethod::Post },
        "put" | "PUT" => quote! { utoipa::openapi::path::HttpMethod::Put },
        "delete" | "DELETE" => quote! { utoipa::openapi::path::HttpMethod::Delete },
        "patch" | "PATCH" => quote! { utoipa::openapi::path::HttpMethod::Patch },
        other => {
            return syn::Error::new(
                method.span(),
                format!("unsupported method `{other}`; use get, post, put, delete, or patch"),
            )
            .to_compile_error()
            .into();
        }
    };

    let request_body = request_ty
        .as_ref()
        .map(|ty| {
            quote! {
                Some(
                    utoipa::openapi::request_body::RequestBodyBuilder::new()
                        .content(
                            "application/json",
                            utoipa::openapi::content::ContentBuilder::new()
                                .schema(Some(<#ty as utoipa::PartialSchema>::schema()))
                                .build(),
                        )
                        .required(Some(utoipa::openapi::Required::True))
                        .build(),
                )
            }
        })
        .unwrap_or_else(|| quote! { None });

    let response_block = response_ty
        .as_ref()
        .map(|ty| {
            quote! {
                responses = responses.response(
                    #status_str,
                    utoipa::openapi::response::ResponseBuilder::new()
                        .description("Success")
                        .content(
                            "application/json",
                            utoipa::openapi::content::ContentBuilder::new()
                                .schema(Some(<#ty as utoipa::PartialSchema>::schema()))
                                .build(),
                        )
                        .build(),
                );
            }
        })
        .unwrap_or_else(|| {
            quote! {
                responses = responses.response(
                    #status_str,
                    utoipa::openapi::response::ResponseBuilder::new()
                        .description("Success")
                        .build(),
                );
            }
        });

    let tags_block = tag
        .as_ref()
        .map(|t| {
            quote! {
                operation.tags = Some(vec![#t.to_string()]);
            }
        })
        .unwrap_or_else(|| quote! {});

    let summary_block = summary
        .as_ref()
        .map(|s| quote! { operation.summary = Some(#s.to_string()); })
        .unwrap_or_else(|| quote! {});

    let description_block = description
        .as_ref()
        .map(|d| quote! { operation.description = Some(#d.to_string()); })
        .unwrap_or_else(|| quote! {});

    let mut schema_types: Vec<Type> = Vec::new();
    if let Some(ty) = request_ty {
        schema_types.push(ty);
    }
    if let Some(ty) = response_ty {
        schema_types.push(ty);
    }

    let schema_push = schema_types.iter().map(|ty| {
        quote! {
            acc.push((<#ty as utoipa::ToSchema>::name().into(), <#ty as utoipa::PartialSchema>::schema()));
            <#ty as utoipa::ToSchema>::schemas(acc);
        }
    });

    let input_fn: syn::ItemFn = parse_macro_input!(item as syn::ItemFn);
    let fn_name = &input_fn.sig.ident;

    let expanded = quote! {
        #input_fn

        #[allow(non_upper_case_globals)]
        const _: () = {
            fn __dy_rs_operation() -> utoipa::openapi::path::Operation {
                let mut responses = utoipa::openapi::ResponsesBuilder::new();
                #response_block

                let mut operation = utoipa::openapi::path::OperationBuilder::new()
                    .operation_id(Some(stringify!(#fn_name)))
                    .responses(responses.build())
                    .request_body(#request_body)
                    .build();

                #tags_block
                #summary_block
                #description_block

                operation
            }

            fn __dy_rs_register_schemas(
                acc: &mut Vec<(String, utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>)>
            ) {
                #(#schema_push)*
            }

            ::dy_rs::openapi::inventory::submit! {
                ::dy_rs::openapi::AutoOperation {
                    path: #path,
                    method: #method_expr,
                    operation: __dy_rs_operation,
                    register_schemas: __dy_rs_register_schemas,
                }
            }
        };
    };

    TokenStream::from(expanded)
}
