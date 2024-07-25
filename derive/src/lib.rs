use minify_html::{minify, Cfg};
use regex::Regex;
use rusty_handlebars_parser::Compiler;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use std::str::FromStr;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, DeriveInput, Ident, Result, LitStr, Token};
use syn::spanned::Spanned;

#[cfg(feature = "minify-html")]
static COMPRESS_CONFIG: Cfg = Cfg {
    minify_js: true,
    minify_css: true,
    do_not_minify_doctype: true,
    ensure_spec_compliant_unquoted_attribute_values: true,
    keep_closing_tags: true,
    keep_html_and_head_opening_tags: true,
    keep_spaces_between_attributes: true,
    keep_comments: false,
    keep_input_type_text_attr: false,
    keep_ssi_comments: false,
    preserve_brace_template_syntax: true,
    preserve_chevron_percent_template_syntax: false,
    remove_bangs: false,
    remove_processing_instructions: false
};

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
    content: Option<proc_macro2::TokenStream>
}

impl Parse for DisplayParts{
    fn parse(input: ParseStream) -> Result<Self> {
        let input = input.parse::<DeriveInput>()?;
        let lifetimes = input.generics.into_token_stream();
        let name = input.ident;
        let attr = match input.attrs.get(0){
            None => return Ok(Self{
                name, generics: lifetimes, content: None
            }),
            Some(attr) => attr
        };
        let args = attr.parse_args::<TemplateArgs>()?;
        let src = match args.src{
            None => return Ok(Self{
                name, generics: lifetimes, content: None
            }),
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
        let buf = minify(buf.as_bytes(), &COMPRESS_CONFIG);
        #[cfg(feature = "minify-html")]   
        let src = unsafe{
            std::str::from_utf8_unchecked(&buf)
        };
        #[cfg(not(feature = "minify-html"))]
        let src = buf.as_str();   
        let content = match Compiler::new().compile(Some("self"), &src){
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
        Ok(Self{
          name, generics: lifetimes,
            content: Some(proc_macro2::token_stream::TokenStream::from_str(&content).unwrap())
        })
    }
}

#[proc_macro_derive(DisplayAsHtml, attributes(Template))]
pub fn make_renderable(raw: TokenStream) -> TokenStream{
    let DisplayParts{
        name, generics, content
    } = parse_macro_input!(raw as DisplayParts);

    let mod_name = proc_macro2::token_stream::TokenStream::from_str((
        format!("{}_to_html_impl", name.to_string().to_lowercase())
    ).as_str()).unwrap();
    let generics_str = generics.to_string();
    let cleaned_generics = proc_macro2::token_stream::TokenStream::from_str(Regex::new(r":[^,>]+").unwrap().replace(&generics_str, "").as_ref()).unwrap();
    TokenStream::from(match content {
        Some(content) => quote! {
            mod #mod_name{
                use std::fmt::Display;
                use rusty_handlebars::{DisplayAsHtml, AsDisplay, AsDisplayHtml, AsBool, as_text, as_html};
                use super::#name;
                impl #generics Display for #name #cleaned_generics {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        #content
                        Ok(())
                    }
                }
                impl #generics DisplayAsHtml for #name #cleaned_generics {}
                impl #generics AsDisplay for #name #cleaned_generics {
                    fn as_display(&self) -> impl Display{
                        self
                    }
                }
                /*impl #lifetimes AsDisplayHtml for #name #lifetimes {
                    fn as_display_html(&self) -> impl Display{
                        self.to_string().as_display_html()
                    }
                }*/
            }
        },
        None => quote! {
            mod #mod_name{
                use std::fmt::Display;
                use rusty_handlebars::DisplayAsHtml;
                use super::#name;
                impl #generics Display for #name #generics {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        Ok(())
                    }
                }
                impl #generics DisplayAsHtml for #name #generics {}
            }
        }
    })
}