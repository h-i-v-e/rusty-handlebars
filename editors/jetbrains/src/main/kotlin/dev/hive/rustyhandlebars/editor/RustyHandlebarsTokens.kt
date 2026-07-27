package dev.hive.rustyhandlebars.editor

import com.intellij.psi.TokenType
import com.intellij.psi.tree.IElementType
import com.intellij.psi.tree.TokenSet
import dev.hive.rustyhandlebars.RustyHandlebarsLanguage

class RustyHandlebarsTokenType(debugName: String) :
    IElementType(debugName, RustyHandlebarsLanguage)

object RustyHandlebarsTokens {
    @JvmField val DATA = RustyHandlebarsTokenType("DATA")
    @JvmField val OPEN = RustyHandlebarsTokenType("OPEN")
    @JvmField val CLOSE = RustyHandlebarsTokenType("CLOSE")
    @JvmField val OPEN_RAW = RustyHandlebarsTokenType("OPEN_RAW")
    @JvmField val CLOSE_RAW = RustyHandlebarsTokenType("CLOSE_RAW")
    @JvmField val COMMENT = RustyHandlebarsTokenType("COMMENT")
    @JvmField val RAW_BLOCK = RustyHandlebarsTokenType("RAW_BLOCK")
    @JvmField val BLOCK_SIGIL = RustyHandlebarsTokenType("BLOCK_SIGIL")
    @JvmField val BUILTIN_BLOCK = RustyHandlebarsTokenType("BUILTIN_BLOCK")
    @JvmField val BUILTIN_HELPER = RustyHandlebarsTokenType("BUILTIN_HELPER")
    @JvmField val KEYWORD = RustyHandlebarsTokenType("KEYWORD")
    @JvmField val PRIVATE_VALUE = RustyHandlebarsTokenType("PRIVATE_VALUE")
    @JvmField val PARENT_PATH = RustyHandlebarsTokenType("PARENT_PATH")
    @JvmField val IDENTIFIER = RustyHandlebarsTokenType("IDENTIFIER")
    @JvmField val STRING = RustyHandlebarsTokenType("STRING")
    @JvmField val NUMBER = RustyHandlebarsTokenType("NUMBER")
    @JvmField val BOOLEAN = RustyHandlebarsTokenType("BOOLEAN")
    @JvmField val PUNCTUATION = RustyHandlebarsTokenType("PUNCTUATION")
    @JvmField val BAD_CHARACTER = TokenType.BAD_CHARACTER

    @JvmField val COMMENTS = TokenSet.create(COMMENT)
    @JvmField val STRINGS = TokenSet.create(STRING, RAW_BLOCK)
}
