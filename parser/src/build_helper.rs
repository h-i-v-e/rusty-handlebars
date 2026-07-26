//! Minifier configuration used by the derive macro.

use minify_html::Cfg;

/// HTML minifier settings that preserve brace-template syntax.
#[cfg(feature = "minify-html")]
pub static COMPRESS_CONFIG: Cfg = Cfg {
    allow_optimal_entities: false,
    minify_js: true,
    minify_css: true,
    minify_doctype: false,
    allow_noncompliant_unquoted_attribute_values: false,
    keep_closing_tags: true,
    keep_html_and_head_opening_tags: true,
    allow_removing_spaces_between_attributes: false,
    keep_comments: false,
    keep_input_type_text_attr: false,
    keep_ssi_comments: false,
    preserve_brace_template_syntax: true,
    preserve_chevron_percent_template_syntax: false,
    remove_bangs: true,
    remove_processing_instructions: false,
};
