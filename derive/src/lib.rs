use minify_html::minify;
use regex::Regex;
use rusty_handlebars_parser::{add_builtins, build_helper, BlockMap, Compiler, Options, USE_AS_DISPLAY};
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use std::env;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, DeriveInput, Ident, Result, LitStr, Token};
use syn::spanned::Spanned;
use toml::Value;

fn find_path() -> PathBuf{
    let path = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).to_path_buf();
    let workspace = match path.parent(){
        None => return path,
        Some(parent) => parent.to_path_buf()
    };
    let cargo = workspace.join("Cargo.toml");
    if !cargo.exists(){
        return path;
    }
    let contents = std::fs::read_to_string(&cargo).map(|contents| Value::from_str(&contents).unwrap()).unwrap();
    let name = path.file_name().unwrap().to_str().unwrap();
    match match contents.get("workspace").and_then(|workspace| workspace.get("members")).and_then(|members| members.as_array()){
        None => return path,
        Some(members) => members.iter().find(|item| item.as_str().unwrap() == name)
    }{
        None => path,
        Some(_) => workspace
    }
}

struct TemplateArgs{
    src: Option<String>
}

impl Parse for TemplateArgs{
    fn parse(input: ParseStream) -> Result<Self> {
        let mut src : Option<String> = None;
        loop {
            let ident = input.parse::<Ident>()?;
            let label = ident.to_string();
            input.parse::<Token!(=)>()?;
            if label == "path"{
                src = Some(input.parse::<LitStr>()?.value());
            }
            if input.is_empty(){
                break;
            }
            input.parse::<Token!(,)>()?;
        }
        Ok(TemplateArgs{
            src
        })
    }
}

struct DisplayParts{
    name: Ident,
    generics: proc_macro2::TokenStream,
    uses: proc_macro2::TokenStream,
    content: proc_macro2::TokenStream
}

impl Parse for DisplayParts{
    fn parse(input: ParseStream) -> Result<Self> {
        let input = input.parse::<DeriveInput>()?;
        let lifetimes = input.generics.into_token_stream();
        let name = input.ident;
        let attr = match input.attrs.get(0){
            None => return Err(
                syn::Error::new(
                    name.span(),
                    "missing template macro"
                )
            ),
            Some(attr) => attr
        };
        let args = attr.parse_args::<TemplateArgs>()?;
        let src = match args.src{
            None => return Err(
                syn::Error::new(
                    attr.span(),
                    "missing path attribute in template macro"
                )
            ),
            Some(src) => src
        };
        let path = find_path().join(src);
        println!("reading {:?}", path);
        let buf = match std::fs::read_to_string(&path){
            Ok(src) => src,
            Err(err) => return Err(
                syn::Error::new(
                    attr.span(),
                    format!(
                        "unable to read {:?}, {}", path, err.to_string()
                    )
                )
            )
        };
        #[cfg(feature = "minify-html")]
        let buf = minify(buf.as_bytes(), &build_helper::COMPRESS_CONFIG);
        #[cfg(feature = "minify-html")]   
        let src = unsafe{
            std::str::from_utf8_unchecked(&buf)
        };
        #[cfg(not(feature = "minify-html"))]
        let src = buf.as_str();
        let mut factories = BlockMap::new();
        add_builtins(&mut factories);
        let mut rust = match Compiler::new(Options{
            write_var_name: "f",
            root_var_name: Some("self")
        }, factories).compile(&src){
            Ok(rust) => rust,
            Err(err) => {
                return Err(
                    syn::Error::new(
                        attr.span(),
                        err.to_string()
                    )
                )
            }
        };
        rust.using.insert("WithRustyHandlebars");
        rust.using.insert(USE_AS_DISPLAY);
        Ok(Self{
            name, generics: lifetimes,
            uses: proc_macro2::token_stream::TokenStream::from_str(&rust.uses().to_string())?,
            content: proc_macro2::token_stream::TokenStream::from_str(&rust.code)?
        })
    }
}

#[proc_macro_derive(WithRustyHandlebars, attributes(template))]
pub fn make_renderable(raw: TokenStream) -> TokenStream{
    let DisplayParts{
        name, generics, uses, content
    } = parse_macro_input!(raw as DisplayParts);

    let mod_name = proc_macro2::token_stream::TokenStream::from_str((
        format!("{}_with_rusty_handlebars_impl", name.to_string().to_lowercase())
    ).as_str()).unwrap();
    let generics_str = generics.to_string();
    let cleaned_generics = proc_macro2::token_stream::TokenStream::from_str(Regex::new(r":[^,>]+").unwrap().replace(&generics_str, "").as_ref()).unwrap();
    TokenStream::from(quote! {
        mod #mod_name{
            use std::fmt::Display;
            #uses;
            use super::#name;
            impl #generics Display for #name #cleaned_generics {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    #content
                    Ok(())
                }
            }
            impl #generics WithRustyHandlebars for #name #cleaned_generics {}
            impl #generics AsDisplay for #name #cleaned_generics {
                fn as_display(&self) -> impl Display{
                    self
                }
            }
        }
    })
}