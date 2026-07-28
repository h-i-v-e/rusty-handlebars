use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use cargo_metadata::{Metadata, MetadataCommand};
use serde::Serialize;
use syn::{punctuated::Punctuated, Expr, Fields, Item, Lit, Meta, Token};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FieldInfo {
    pub name: String,
    pub ty: String,
    pub source: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TemplateContext {
    pub name: String,
    pub template: PathBuf,
    pub helpers: Vec<String>,
    pub fields: Vec<FieldInfo>,
}

#[derive(Debug, Default)]
pub struct ProjectIndex {
    contexts: HashMap<PathBuf, Vec<TemplateContext>>,
}

impl ProjectIndex {
    pub fn discover(root: &Path) -> Result<Self, String> {
        let metadata = MetadataCommand::new()
            .current_dir(root)
            .no_deps()
            .exec()
            .map_err(|error| format!("cargo metadata failed: {error}"))?;
        Self::from_metadata(&metadata)
    }

    fn from_metadata(metadata: &Metadata) -> Result<Self, String> {
        let workspace_root = metadata.workspace_root.as_std_path();
        let mut index = Self::default();
        for package in metadata.packages.iter().filter(|package| {
            package
                .manifest_path
                .as_std_path()
                .starts_with(workspace_root)
        }) {
            let manifest_dir = package
                .manifest_path
                .parent()
                .map(|path| path.as_std_path().to_path_buf())
                .ok_or_else(|| format!("manifest has no parent: {}", package.manifest_path))?;
            let template_root = PathBuf::from(workspace_root);
            let source_root = manifest_dir.join("src");
            let mut source_files = Vec::new();
            collect_rust_files(&source_root, &mut source_files)?;
            for source in source_files {
                index.index_source(&source, &template_root)?;
            }
        }
        let root_source = workspace_root.join("src");
        if root_source.exists() {
            let mut source_files = Vec::new();
            collect_rust_files(&root_source, &mut source_files)?;
            for source in source_files {
                index.index_source(&source, workspace_root)?;
            }
        }
        Ok(index)
    }

    pub fn contexts_for(&self, template: &Path) -> &[TemplateContext] {
        self.contexts
            .get(&normalize_path(template))
            .map_or(&[], Vec::as_slice)
    }

    fn index_source(&mut self, source_path: &Path, template_root: &Path) -> Result<(), String> {
        let source = fs::read_to_string(source_path)
            .map_err(|error| format!("unable to read {}: {error}", source_path.display()))?;
        let syntax = match syn::parse_file(&source) {
            Ok(syntax) => syntax,
            Err(_) => return Ok(()),
        };
        self.index_items(syntax.items, source_path, template_root);
        Ok(())
    }

    fn index_items(&mut self, items: Vec<Item>, source_path: &Path, template_root: &Path) {
        for item in items {
            let item = match item {
                Item::Struct(item) => item,
                Item::Mod(module) => {
                    if let Some((_, items)) = module.content {
                        self.index_items(items, source_path, template_root);
                    }
                    continue;
                }
                _ => continue,
            };
            if !derives_rusty_handlebars(&item.attrs) {
                continue;
            }
            let Some(attribute) = item
                .attrs
                .iter()
                .find(|attribute| attribute.path().is_ident("template"))
            else {
                continue;
            };
            let Ok(arguments) =
                attribute.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            else {
                continue;
            };
            let Some(template_path) = template_path(&arguments) else {
                continue;
            };
            let template = normalize_path(&template_root.join(template_path));
            let fields = match item.fields {
                Fields::Named(fields) => fields
                    .named
                    .into_iter()
                    .filter_map(|field| {
                        Some(FieldInfo {
                            name: field.ident?.to_string(),
                            ty: quote_type(&field.ty),
                            source: source_path.to_path_buf(),
                        })
                    })
                    .collect(),
                _ => Vec::new(),
            };
            self.contexts
                .entry(template.clone())
                .or_default()
                .push(TemplateContext {
                    name: item.ident.to_string(),
                    template,
                    helpers: helper_paths(&arguments),
                    fields,
                });
        }
    }
}

fn derives_rusty_handlebars(attributes: &[syn::Attribute]) -> bool {
    attributes
        .iter()
        .filter(|attribute| attribute.path().is_ident("derive"))
        .any(|attribute| {
            attribute
                .parse_args_with(Punctuated::<syn::Path, Token![,]>::parse_terminated)
                .is_ok_and(|paths| {
                    paths.iter().any(|path| {
                        path.segments
                            .last()
                            .is_some_and(|segment| segment.ident == "WithRustyHandlebars")
                    })
                })
        })
}

fn template_path(arguments: &Punctuated<Meta, Token![,]>) -> Option<String> {
    arguments.iter().find_map(|argument| {
        let Meta::NameValue(value) = argument else {
            return None;
        };
        if !value.path.is_ident("path") {
            return None;
        }
        let Expr::Lit(expression) = &value.value else {
            return None;
        };
        let Lit::Str(value) = &expression.lit else {
            return None;
        };
        Some(value.value())
    })
}

fn helper_paths(arguments: &Punctuated<Meta, Token![,]>) -> Vec<String> {
    arguments
        .iter()
        .find_map(|argument| {
            let Meta::NameValue(value) = argument else {
                return None;
            };
            if !value.path.is_ident("helpers") {
                return None;
            }
            let Expr::Array(values) = &value.value else {
                return None;
            };
            Some(
                values
                    .elems
                    .iter()
                    .filter_map(|expression| {
                        let Expr::Lit(expression) = expression else {
                            return None;
                        };
                        let Lit::Str(value) = &expression.lit else {
                            return None;
                        };
                        Some(value.value())
                    })
                    .collect(),
            )
        })
        .unwrap_or_default()
}

fn quote_type(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(path) => path
            .path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>()
            .join("::"),
        syn::Type::Reference(reference) => format!("&{}", quote_type(&reference.elem)),
        syn::Type::Slice(slice) => format!("[{}]", quote_type(&slice.elem)),
        syn::Type::Array(array) => format!("[{}; _]", quote_type(&array.elem)),
        syn::Type::Tuple(tuple) => format!(
            "({})",
            tuple
                .elems
                .iter()
                .map(quote_type)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        _ => "<unresolved>".to_owned(),
    }
}

fn collect_rust_files(directory: &Path, output: &mut Vec<PathBuf>) -> Result<(), String> {
    if !directory.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(directory)
        .map_err(|error| format!("unable to read {}: {error}", directory.display()))?
    {
        let path = entry
            .map_err(|error| format!("unable to read directory entry: {error}"))?
            .path();
        if path.is_dir() {
            collect_rust_files(&path, output)?;
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            output.push(path);
        }
    }
    Ok(())
}

fn normalize_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_template_attribute_values() {
        let arguments: Punctuated<Meta, Token![,]> = syn::parse::Parser::parse_str(
            Punctuated::<Meta, Token![,]>::parse_terminated,
            r#"path = "templates/page.rhbs", helpers = ["crate::title"]"#,
        )
        .unwrap();
        assert_eq!(
            template_path(&arguments).as_deref(),
            Some("templates/page.rhbs")
        );
        assert_eq!(helper_paths(&arguments), ["crate::title"]);
    }

    #[test]
    fn discovers_contexts_in_the_workspace() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("language-server is a workspace member");
        let index = ProjectIndex::discover(root).unwrap();
        let contexts = index.contexts_for(&root.join("examples/templates/hello-world.rhbs"));
        assert!(
            contexts.iter().any(|context| {
                context.name == "TestTemplate"
                    && context.fields.iter().any(|field| field.name == "message")
            }),
            "{contexts:?}"
        );
    }
}
