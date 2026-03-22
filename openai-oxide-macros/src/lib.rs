use proc_macro::TokenStream;
use quote::quote;
use syn::{FnArg, ItemFn, LitStr, Pat, Type, parse_macro_input};

/// Automatically generates a JSON Schema tool definition for an OpenAI function tool.
///
/// Generates a companion function `<function_name>_tool()` that returns a `serde_json::Value`
/// representing the `ChatCompletionToolParam`.
#[proc_macro_attribute]
pub fn openai_tool(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut description = String::new();

    // Parse the attribute arguments to extract the description
    let attr_parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("description") {
            let lit: LitStr = meta.value()?.parse()?;
            description = lit.value();
            Ok(())
        } else {
            Err(meta.error("unsupported property"))
        }
    });
    parse_macro_input!(attr with attr_parser);

    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let fn_name_str = fn_name.to_string();
    let tool_fn_name = syn::Ident::new(&format!("{}_tool", fn_name_str), fn_name.span());

    let mut properties_keys = Vec::new();
    let mut properties_values = Vec::new();
    let mut required = Vec::new();

    for arg in &input_fn.sig.inputs {
        if let FnArg::Typed(pat_type) = arg {
            if let Pat::Ident(pat_ident) = &*pat_type.pat {
                let arg_name = pat_ident.ident.to_string();
                let is_option = is_type_option(&pat_type.ty);

                let arg_type_str = map_rust_type_to_json(&pat_type.ty);

                properties_keys.push(arg_name.clone());
                properties_values.push(quote! {
                    serde_json::json!({
                        "type": #arg_type_str
                    })
                });

                if !is_option {
                    required.push(arg_name);
                }
            }
        }
    }

    let expanded = quote! {
        // Output the original function intact
        #input_fn

        // Output the generated tool definition function
        pub fn #tool_fn_name() -> serde_json::Value {
            let mut props = serde_json::Map::new();
            #(
                props.insert(#properties_keys.to_string(), #properties_values);
            )*

            serde_json::json!({
                "type": "function",
                "function": {
                    "name": #fn_name_str,
                    "description": #description,
                    "parameters": {
                        "type": "object",
                        "properties": props,
                        "required": vec![ #( #required ),* ]
                    }
                }
            })
        }
    };

    TokenStream::from(expanded)
}

fn is_type_option(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}

fn map_rust_type_to_json(ty: &Type) -> String {
    let mut ty_str = "string".to_string();
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let ident = segment.ident.to_string();
            ty_str = match ident.as_str() {
                "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "usize" | "isize" => {
                    "integer".to_string()
                }
                "f32" | "f64" => "number".to_string(),
                "bool" => "boolean".to_string(),
                "Option" => {
                    // Extract inner type if possible, fallback to string
                    "string".to_string()
                }
                _ => "string".to_string(),
            };
        }
    }
    ty_str
}
