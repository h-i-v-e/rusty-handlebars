use minify_html::minify;
use regex::Regex;
use rusty_handlebars_parser::{add_builtins, build_helper, BlockMap, Compiler, Options, USE_AS_DISPLAY};
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use std::str::FromStr;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, DeriveInput, Ident, Result, LitStr, Token};
use syn::spanned::Spanned;

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
                    "missing Template macro"
                )
            ),
            Some(attr) => attr
        };
        let args = attr.parse_args::<TemplateArgs>()?;
        let src = match args.src{
            None => return Err(
                syn::Error::new(
                    attr.span(),
                    "missing path attribute in Template macro"
                )
            ),
            Some(src) => src
        };
        let buf = match std::fs::read_to_string(&src) {
            Ok(src) => src,
            Err(err) => {
                let path = std::fs::canonicalize(std::path::Path::new("./")).unwrap();
                return Err(
                    syn::Error::new(
                        attr.span(),
                        format!(
                            "unable to read {}, {}", path.join(src).to_str().unwrap(), err.to_string()
                        )
                    )
                )
            }
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

#[proc_macro_derive(WithRustyHandlebars, attributes(Template))]
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