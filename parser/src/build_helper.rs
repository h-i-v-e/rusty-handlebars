//! HTML minification configuration
//!
//! This module provides configuration for HTML minification when the `minify-html` feature
//! is enabled. It uses the `minify-html` crate to optimize HTML output while preserving
//! template syntax.

use minify_html::Cfg;

/// Configuration for HTML minification
///
/// This configuration is used when the `minify-html` feature is enabled to optimize
/// HTML output. It's designed to:
///
/// - Preserve Handlebars template syntax
/// - Maintain HTML validity
/// - Optimize JavaScript and CSS
/// - Keep essential HTML structure
#[cfg(feature = "minify-html")]
pub static COMPRESS_CONFIG: Cfg = Cfg {
    // Minify JavaScript in script tags
    minify_js: true,
    // Minify CSS in style tags
    minify_css: true,
    // Preserve doctype declarations
    do_not_minify_doctype: true,
    // Ensure attribute values are spec-compliant
    ensure_spec_compliant_unquoted_attribute_values: true,
    // Keep closing tags for elements that require them
    keep_closing_tags: true,
    // Preserve html and head opening tags
    keep_html_and_head_opening_tags: true,
    // Maintain spaces between attributes
    keep_spaces_between_attributes: true,
    // Remove HTML comments
    keep_comments: false,
    // Remove type="text" from input elements
    keep_input_type_text_attr: false,
    // Remove SSI comments
    keep_ssi_comments: false,
    // Preserve Handlebars template syntax
    preserve_brace_template_syntax: true,
    // Remove ASP-style template syntax
    preserve_chevron_percent_template_syntax: false,
    // Remove HTML comments
    remove_bangs: false,
    // Remove XML processing instructions
    remove_processing_instructions: false
};