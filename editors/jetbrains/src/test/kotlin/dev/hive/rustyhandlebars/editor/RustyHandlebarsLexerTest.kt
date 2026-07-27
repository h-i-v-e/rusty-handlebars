package dev.hive.rustyhandlebars.editor

import com.intellij.psi.tree.IElementType
import kotlin.test.Test
import kotlin.test.assertEquals

class RustyHandlebarsLexerTest {
    @Test
    fun tokenizesHtmlAndExpressions() {
        assertEquals(
            listOf(
                RustyHandlebarsTokens.DATA to "<p>",
                RustyHandlebarsTokens.OPEN to "{{",
                RustyHandlebarsTokens.BUILTIN_BLOCK to "if",
                com.intellij.psi.TokenType.WHITE_SPACE to " ",
                RustyHandlebarsTokens.IDENTIFIER to "user.name",
                RustyHandlebarsTokens.CLOSE to "}}",
                RustyHandlebarsTokens.DATA to "</p>",
            ),
            tokens("<p>{{if user.name}}</p>"),
        )
    }

    @Test
    fun preservesCommentsRawBlocksAndEscapedOpenings() {
        assertEquals(
            listOf(
                RustyHandlebarsTokens.DATA to "\\{{literal}} ",
                RustyHandlebarsTokens.COMMENT to "{{~!-- note --~}}",
                RustyHandlebarsTokens.DATA to " ",
                RustyHandlebarsTokens.RAW_BLOCK to
                    "{{{{raw}}}}{{not_an_expression}}{{{{/raw}}}}",
            ),
            tokens(
                "\\{{literal}} {{~!-- note --~}} " +
                    "{{{{raw}}}}{{not_an_expression}}{{{{/raw}}}}",
            ),
        )
    }

    @Test
    fun keepsUnterminatedInputTokenizable() {
        assertEquals(
            listOf(
                RustyHandlebarsTokens.OPEN to "{{",
                RustyHandlebarsTokens.BUILTIN_HELPER to "format",
                com.intellij.psi.TokenType.WHITE_SPACE to " ",
                RustyHandlebarsTokens.STRING to "\"unterminated",
            ),
            tokens("{{format \"unterminated"),
        )
    }

    private fun tokens(source: String): List<Pair<IElementType, String>> {
        val lexer = RustyHandlebarsLexer()
        lexer.start(source)
        return buildList {
            while (lexer.tokenType != null) {
                add(
                    lexer.tokenType!! to
                        source.substring(lexer.tokenStart, lexer.tokenEnd),
                )
                lexer.advance()
            }
        }
    }
}
