use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::{parse_macro_input, LitStr};

#[proc_macro]
pub fn i18n_assets(input: TokenStream) -> TokenStream {
    let dir = parse_macro_input!(input as LitStr).value();
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("Failed to get manifest dir");

    let dir_path = std::path::Path::new(&manifest_dir).join(&dir);
    if !dir_path.exists() || !dir_path.is_dir() {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("i18n directory not found: {}", dir_path.display()),
        )
        .to_compile_error()
        .into();
    }

    let mut lang_initializers = Vec::new();
    let mut tracked_files = Vec::new();

    // 搜索文件
    let full_pattern = format!("{}/{}/*.toml", manifest_dir, dir);
    for entry in glob::glob(&full_pattern).expect("Failed to read glob") {
        let path = entry.expect("Path error");
        
        // Skip if it is a directory
        if path.is_dir() {
            continue;
        }

        let lang_code = path.file_stem().unwrap().to_str().unwrap().to_string();

        // 1. 强制编译器监视文件（解决修改 TOML 不触发重新编译的问题）
        let abs_path = path.canonicalize().unwrap();
        let abs_path_str = abs_path.to_str().unwrap();
        tracked_files.push(quote! { const _: &[u8] = include_bytes!(#abs_path_str); });

        // 2. 解析 TOML
        let content = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("Read error for path {:?}: {}", path, e));
        let kv: HashMap<String, String> = toml::from_str(&content).expect("TOML error");

        let mut key_inserts = Vec::new();
        for (key, val) in kv {
            let parts = parse_template(&val);
            let part_tokens = parts.iter().map(|p| match p {
                TemplatePart::Static(s) => quote! { rivus_axum::I18nPart::Static(#s) },
                TemplatePart::Placeholder(p) => quote! { rivus_axum::I18nPart::Placeholder(#p) },
            });

            key_inserts.push(quote! {
                inner_map.insert(#key, vec![ #(#part_tokens),* ]);
            });
        }

        lang_initializers.push(quote! {
            let mut inner_map = std::collections::HashMap::new();
            #(#key_inserts)*
            master_map.insert(#lang_code, inner_map);
        });
    }

    quote! {
        #(#tracked_files)*

        #[rivus_axum::ctor::ctor(crate_path = ::rivus_axum::ctor)]
        fn auto_init_i18n() {
            let mut master_map = std::collections::HashMap::new();
            #(#lang_initializers)*
            rivus_axum::internal_init_i18n(master_map);
        }
    }.into()
}

#[derive(Debug, PartialEq)]
enum TemplatePart { Static(String), Placeholder(String) }

fn parse_template(mut input: &str) -> Vec<TemplatePart> {
    let mut parts = Vec::new();
    while let Some(start) = input.find('{') {
        if start > 0 { parts.push(TemplatePart::Static(input[..start].to_string())); }
        input = &input[start + 1..];
        if let Some(end) = input.find('}') {
            parts.push(TemplatePart::Placeholder(input[..end].to_string()));
            input = &input[end + 1..];
        }
    }
    if !input.is_empty() { parts.push(TemplatePart::Static(input.to_string())); }
    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_template() {
        // Plain text
        assert_eq!(
            parse_template("Hello world"),
            vec![TemplatePart::Static("Hello world".to_string())]
        );

        // Single placeholder
        assert_eq!(
            parse_template("Hello {name}!"),
            vec![
                TemplatePart::Static("Hello ".to_string()),
                TemplatePart::Placeholder("name".to_string()),
                TemplatePart::Static("!".to_string())
            ]
        );

        // Multiple placeholders
        assert_eq!(
            parse_template("{first} and {second}"),
            vec![
                TemplatePart::Placeholder("first".to_string()),
                TemplatePart::Static(" and ".to_string()),
                TemplatePart::Placeholder("second".to_string())
            ]
        );

        // Placeholder at start
        assert_eq!(
            parse_template("{name} hello"),
            vec![
                TemplatePart::Placeholder("name".to_string()),
                TemplatePart::Static(" hello".to_string())
            ]
        );

        // Placeholder at end
        assert_eq!(
            parse_template("hello {name}"),
            vec![
                TemplatePart::Static("hello ".to_string()),
                TemplatePart::Placeholder("name".to_string())
            ]
        );

        // Empty string
        assert_eq!(parse_template(""), vec![]);
        
        // Unclosed brace (current implementation might treat it weirdly or ignore, let's check logic)
        // input.find('{') finds it. start > 0 maybe. 
        // input = input[start+1..]
        // input.find('}') -> None.
        // loop terminates.
        // pushes remainder as static.
        assert_eq!(
            parse_template("Hello {name"),
            vec![
                TemplatePart::Static("Hello ".to_string()), 
                TemplatePart::Static("name".to_string())
            ]
        );
    }
}
