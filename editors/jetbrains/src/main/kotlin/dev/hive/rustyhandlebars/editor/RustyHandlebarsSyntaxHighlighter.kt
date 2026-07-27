package dev.hive.rustyhandlebars.editor

import com.intellij.ide.highlighter.HtmlFileHighlighter
import com.intellij.lexer.LayeredLexer
import com.intellij.lexer.Lexer
import com.intellij.openapi.editor.DefaultLanguageHighlighterColors
import com.intellij.openapi.editor.HighlighterColors
import com.intellij.openapi.editor.colors.TextAttributesKey
import com.intellij.openapi.fileTypes.SyntaxHighlighterBase
import com.intellij.psi.tree.IElementType

class RustyHandlebarsSyntaxHighlighter : SyntaxHighlighterBase() {
    private val htmlHighlighter = HtmlFileHighlighter()

    override fun getHighlightingLexer(): Lexer =
        LayeredLexer(RustyHandlebarsLexer()).apply {
            registerLayer(
                HtmlFileHighlighter().highlightingLexer,
                RustyHandlebarsTokens.DATA,
            )
        }

    override fun getTokenHighlights(tokenType: IElementType): Array<TextAttributesKey> {
        val attribute = ATTRIBUTES[tokenType]
        if (attribute != null) return pack(attribute)
        @Suppress("UNCHECKED_CAST")
        return htmlHighlighter.getTokenHighlights(tokenType) as Array<TextAttributesKey>
    }

    companion object {
        private val DELIMITER = TextAttributesKey.createTextAttributesKey(
            "RUSTY_HANDLEBARS_DELIMITER",
            DefaultLanguageHighlighterColors.BRACES,
        )
        private val BLOCK = TextAttributesKey.createTextAttributesKey(
            "RUSTY_HANDLEBARS_BLOCK",
            DefaultLanguageHighlighterColors.KEYWORD,
        )
        private val HELPER = TextAttributesKey.createTextAttributesKey(
            "RUSTY_HANDLEBARS_HELPER",
            DefaultLanguageHighlighterColors.FUNCTION_CALL,
        )
        private val KEYWORD = TextAttributesKey.createTextAttributesKey(
            "RUSTY_HANDLEBARS_KEYWORD",
            DefaultLanguageHighlighterColors.KEYWORD,
        )
        private val VARIABLE = TextAttributesKey.createTextAttributesKey(
            "RUSTY_HANDLEBARS_VARIABLE",
            DefaultLanguageHighlighterColors.IDENTIFIER,
        )
        private val PRIVATE = TextAttributesKey.createTextAttributesKey(
            "RUSTY_HANDLEBARS_PRIVATE",
            DefaultLanguageHighlighterColors.INSTANCE_FIELD,
        )
        private val STRING = TextAttributesKey.createTextAttributesKey(
            "RUSTY_HANDLEBARS_STRING",
            DefaultLanguageHighlighterColors.STRING,
        )
        private val NUMBER = TextAttributesKey.createTextAttributesKey(
            "RUSTY_HANDLEBARS_NUMBER",
            DefaultLanguageHighlighterColors.NUMBER,
        )
        private val COMMENT = TextAttributesKey.createTextAttributesKey(
            "RUSTY_HANDLEBARS_COMMENT",
            DefaultLanguageHighlighterColors.BLOCK_COMMENT,
        )
        private val BAD = TextAttributesKey.createTextAttributesKey(
            "RUSTY_HANDLEBARS_BAD_CHARACTER",
            HighlighterColors.BAD_CHARACTER,
        )

        private val ATTRIBUTES = mapOf(
            RustyHandlebarsTokens.OPEN to DELIMITER,
            RustyHandlebarsTokens.CLOSE to DELIMITER,
            RustyHandlebarsTokens.OPEN_RAW to DELIMITER,
            RustyHandlebarsTokens.CLOSE_RAW to DELIMITER,
            RustyHandlebarsTokens.BLOCK_SIGIL to BLOCK,
            RustyHandlebarsTokens.BUILTIN_BLOCK to BLOCK,
            RustyHandlebarsTokens.BUILTIN_HELPER to HELPER,
            RustyHandlebarsTokens.KEYWORD to KEYWORD,
            RustyHandlebarsTokens.IDENTIFIER to VARIABLE,
            RustyHandlebarsTokens.PRIVATE_VALUE to PRIVATE,
            RustyHandlebarsTokens.PARENT_PATH to KEYWORD,
            RustyHandlebarsTokens.STRING to STRING,
            RustyHandlebarsTokens.RAW_BLOCK to STRING,
            RustyHandlebarsTokens.NUMBER to NUMBER,
            RustyHandlebarsTokens.BOOLEAN to NUMBER,
            RustyHandlebarsTokens.COMMENT to COMMENT,
            RustyHandlebarsTokens.BAD_CHARACTER to BAD,
        )
    }
}
