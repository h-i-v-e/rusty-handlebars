//! Implementation of the `rusty-handlebars` derive macro.
//!
//! Applications normally use the macro re-exported by the
//! `rusty-handlebars` facade crate.

#[cfg(feature = "minify-html")]
use minify_html::minify;
use proc_macro::TokenStream;
use quote::quote;
use rusty_handlebars_parser::{add_builtins, BlockMap, Compiler, Options};
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::OnceLock;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{parse_macro_input, DeriveInput, Generics, Ident, LitBool, LitStr, Result, Token};
use toml::Table;

fn discover_path() -> PathBuf {
    let path = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).to_path_buf();
    let mut name = path.file_name().unwrap().to_str().unwrap().to_string();
    let mut local = path.clone();
    loop {
        let workspace = match local.parent() {
            None => return path,
            Some(parent) => parent.to_path_buf(),
        };
        let cargo = workspace.join("Cargo.toml");
        if cargo.exists() {
            let contents: Table = std::fs::read_to_string(&cargo)
                .map(|contents| contents.parse().unwrap())
                .unwrap();
            if let Some(members) = contents
                .get("workspace")
                .and_then(|workspace| workspace.get("members"))
                .and_then(|members| members.as_array())
            {
                if members
                    .iter()
                    .any(|item| item.as_str() == Some(name.as_str()))
                {
                    return workspace;
                }
            }
        }
        name = match workspace.file_name() {
            None => return path,
            Some(base) => format!("{}/{}", base.to_str().unwrap(), name),
        };
        local = workspace;
        continue;
    }
}

fn find_path() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(discover_path)
}

#[cfg(feature = "minify-html")]
fn minify_template(src: String, enabled: bool) -> String {
    if enabled {
        String::from_utf8(minify(
            src.as_bytes(),
            &rusty_handlebars_parser::build_helper::COMPRESS_CONFIG,
        ))
        .expect("minify-html returned invalid UTF-8 for a UTF-8 template")
    } else {
        src
    }
}

#[cfg(not(feature = "minify-html"))]
fn minify_template(src: String, _enabled: bool) -> String {
    src
}

struct TemplateArgs {
    src: Option<String>,
    helpers: Vec<String>,
    minify: bool,
}

fn parse_helpers(input: ParseStream, helpers: &mut Vec<String>) -> Result<()> {
    let content;
    syn::bracketed!(content in input);
    helpers.extend(
        syn::punctuated::Punctuated::<LitStr, Token![,]>::parse_terminated(&content)?
            .into_iter()
            .map(|helper| helper.value()),
    );
    Ok(())
}

fn helper_paths(helpers: Vec<String>) -> HashMap<String, String> {
    helpers
        .into_iter()
        .map(|helper| {
            let name = helper
                .rsplit("::")
                .next()
                .unwrap_or(helper.as_str())
                .to_string();
            let path = if helper.starts_with("::")
                || helper.starts_with("crate::")
                || helper.starts_with("self::")
                || helper.starts_with("super::")
            {
                helper
            } else {
                format!("::rusty_handlebars::{}", helper)
            };
            (name, path)
        })
        .collect()
}

impl Parse for TemplateArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut src: Option<String> = None;
        let mut minify = true;
        let mut helpers = Vec::<String>::new();
        loop {
            let ident = input.parse::<Ident>()?;
            let label = ident.to_string();
            input.parse::<Token!(=)>()?;
            match label.as_str() {
                "minify" => minify = input.parse::<LitBool>()?.value(),
                "path" => src = Some(input.parse::<LitStr>()?.value()),
                "helpers" => parse_helpers(input, &mut helpers)?,
                _ => {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!("unknown attribute {}", label),
                    ))
                }
            }
            if input.is_empty() {
                break;
            }
            input.parse::<Token!(,)>()?;
        }
        Ok(TemplateArgs {
            src,
            helpers,
            minify,
        })
    }
}

struct DisplayParts {
    name: Ident,
    generics: Generics,
    content: proc_macro2::TokenStream,
}

impl Parse for DisplayParts {
    fn parse(input: ParseStream) -> Result<Self> {
        let input = input.parse::<DeriveInput>()?;
        let generics = input.generics;
        let name = input.ident;
        let attr = match input
            .attrs
            .iter()
            .find(|attribute| attribute.path().is_ident("template"))
        {
            None => return Err(syn::Error::new(name.span(), "missing template macro")),
            Some(attr) => attr,
        };
        let args = attr.parse_args::<TemplateArgs>()?;
        let src = match args.src {
            None => {
                return Err(syn::Error::new(
                    attr.span(),
                    "missing path attribute in template macro",
                ))
            }
            Some(src) => src,
        };
        let path = find_path().join(src);
        let buf = match std::fs::read_to_string(&path) {
            Ok(src) => src,
            Err(err) => {
                return Err(syn::Error::new(
                    attr.span(),
                    format!("unable to read {path:?}, {err}"),
                ))
            }
        };
        let buf = minify_template(buf, args.minify);
        let mut factories = BlockMap::new();
        add_builtins(&mut factories);
        let rust = match Compiler::new(
            Options {
                write_var_name: "f",
                root_var_name: Some("self"),
            },
            factories,
        )
        .with_helper_paths(helper_paths(args.helpers))
        .compile(&buf)
        {
            Ok(rust) => rust,
            Err(err) => return Err(syn::Error::new(attr.span(), err.to_string())),
        };
        Ok(Self {
            name,
            generics,
            content: proc_macro2::token_stream::TokenStream::from_str(&rust.code)?,
        })
    }
}

/// Implements template rendering for a struct.
///
/// `#[template(path = "...")]` names the template file. `minify = false`
/// disables the default HTML minification. `helpers = ["crate::helper"]`
/// maps an inline helper's final path segment to that Rust function path.
///
/// The generated implementations are `std::fmt::Display`,
/// `rusty_handlebars::WithRustyHandlebars`, and
/// `rusty_handlebars::AsDisplay`.
#[proc_macro_derive(WithRustyHandlebars, attributes(template))]
pub fn make_renderable(raw: TokenStream) -> TokenStream {
    let DisplayParts {
        name,
        generics,
        content,
    } = parse_macro_input!(raw as DisplayParts);

    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    TokenStream::from(quote! {
        impl #impl_generics ::std::fmt::Display for #name #type_generics #where_clause {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                #content
                Ok(())
            }
        }
        impl #impl_generics ::rusty_handlebars::WithRustyHandlebars for #name #type_generics #where_clause {}
        impl #impl_generics ::rusty_handlebars::AsDisplay for #name #type_generics #where_clause {
            fn as_display(&self) -> impl ::std::fmt::Display {
                self
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use crate::{find_path, helper_paths, TemplateArgs};

    #[test]
    fn test_find() {
        println!("{:?}", find_path());
    }

    #[test]
    fn parses_and_qualifies_helpers() {
        let args: TemplateArgs = syn::parse_str(
            r#"path = "template.hbs", helpers = ["format_date", "crate::capitalize"]"#,
        )
        .unwrap();
        let paths = helper_paths(args.helpers);
        assert_eq!(paths["format_date"], "::rusty_handlebars::format_date");
        assert_eq!(paths["capitalize"], "crate::capitalize");
    }
}
