package dev.hive.rustyhandlebars.editor

import com.intellij.lexer.LexerBase
import com.intellij.psi.TokenType
import com.intellij.psi.tree.IElementType

class RustyHandlebarsLexer : LexerBase() {
    private var buffer: CharSequence = ""
    private var endOffset = 0
    private var tokenStart = 0
    private var tokenEnd = 0
    private var tokenType: IElementType? = null
    private var state = DATA_STATE

    override fun start(
        buffer: CharSequence,
        startOffset: Int,
        endOffset: Int,
        initialState: Int,
    ) {
        this.buffer = buffer
        this.endOffset = endOffset
        tokenStart = startOffset
        tokenEnd = startOffset
        state = initialState
        locateToken()
    }

    override fun getState() = state
    override fun getTokenType() = tokenType
    override fun getTokenStart() = tokenStart
    override fun getTokenEnd() = tokenEnd
    override fun getBufferSequence() = buffer
    override fun getBufferEnd() = endOffset

    override fun advance() {
        tokenStart = tokenEnd
        locateToken()
    }

    private fun locateToken() {
        if (tokenStart >= endOffset) {
            tokenType = null
            tokenEnd = endOffset
            return
        }

        if (state == DATA_STATE) {
            lexData()
        } else {
            lexExpression()
        }
    }

    private fun lexData() {
        val opening = nextOpening(tokenStart)
        if (opening < 0) {
            tokenType = RustyHandlebarsTokens.DATA
            tokenEnd = endOffset
            return
        }
        if (opening > tokenStart) {
            tokenType = RustyHandlebarsTokens.DATA
            tokenEnd = opening
            return
        }

        when {
            startsWith("{{!--") || startsWith("{{~!--") -> {
                tokenType = RustyHandlebarsTokens.COMMENT
                tokenEnd = endOfEither("--}}", "--~}}", tokenStart + 6)
            }
            startsWith("{{!") || startsWith("{{~!") -> {
                tokenType = RustyHandlebarsTokens.COMMENT
                tokenEnd = endOf("}}", tokenStart + 3)
            }
            startsWith("{{{{") -> {
                lexRawBlock()
            }
            startsWith("{{{") -> {
                tokenType = RustyHandlebarsTokens.OPEN_RAW
                tokenEnd = consumeTrim(tokenStart + 3)
                state = EXPRESSION_STATE
            }
            else -> {
                tokenType = RustyHandlebarsTokens.OPEN
                tokenEnd = consumeTrim(tokenStart + 2)
                state = EXPRESSION_STATE
            }
        }
    }

    private fun lexExpression() {
        when {
            startsWith("}}}}") -> close(RustyHandlebarsTokens.CLOSE_RAW, 4)
            startsWith("}}}") -> close(RustyHandlebarsTokens.CLOSE_RAW, 3)
            startsWith("}}") -> close(RustyHandlebarsTokens.CLOSE, 2)
            startsWith("~}}}}") -> close(RustyHandlebarsTokens.CLOSE_RAW, 5)
            startsWith("~}}}") -> close(RustyHandlebarsTokens.CLOSE_RAW, 4)
            startsWith("~}}") -> close(RustyHandlebarsTokens.CLOSE, 3)
            buffer[tokenStart].isWhitespace() -> consumeWhile(TokenType.WHITE_SPACE) {
                it.isWhitespace()
            }
            buffer[tokenStart] == '"' -> lexString()
            buffer[tokenStart] == '#' || buffer[tokenStart] == '/' -> {
                tokenType = RustyHandlebarsTokens.BLOCK_SIGIL
                tokenEnd = tokenStart + 1
            }
            startsWith("../") -> {
                tokenType = RustyHandlebarsTokens.PARENT_PATH
                tokenEnd = tokenStart + 3
            }
            buffer[tokenStart] == '@' -> lexPrivateValue()
            buffer[tokenStart].isDigit() -> consumeWhile(RustyHandlebarsTokens.NUMBER) {
                it.isDigit() || it == '.'
            }
            buffer[tokenStart].isIdentifierStart() -> lexWord()
            buffer[tokenStart] in "|().," -> {
                tokenType = RustyHandlebarsTokens.PUNCTUATION
                tokenEnd = tokenStart + 1
            }
            else -> {
                tokenType = RustyHandlebarsTokens.BAD_CHARACTER
                tokenEnd = tokenStart + 1
            }
        }
    }

    private fun lexString() {
        var offset = tokenStart + 1
        var escaped = false
        while (offset < endOffset) {
            val character = buffer[offset++]
            if (character == '"' && !escaped) break
            escaped = character == '\\' && !escaped
            if (character != '\\') escaped = false
        }
        tokenType = RustyHandlebarsTokens.STRING
        tokenEnd = offset
    }

    private fun lexRawBlock() {
        val openingEnd = buffer.indexOf("}}}}", tokenStart + 4, endOffset)
        if (openingEnd < 0) {
            tokenType = RustyHandlebarsTokens.RAW_BLOCK
            tokenEnd = endOffset
            return
        }
        val name = buffer.subSequence(tokenStart + 4, openingEnd)
            .toString()
            .trim(' ', '\t', '\r', '\n', '~')
        val closing = if (name.isEmpty()) null else "{{{{/$name}}}}"
        val closingStart = closing?.let {
            buffer.indexOf(it, openingEnd + 4, endOffset)
        } ?: -1
        tokenType = RustyHandlebarsTokens.RAW_BLOCK
        tokenEnd = if (closingStart < 0) {
            openingEnd + 4
        } else {
            closingStart + closing!!.length
        }
    }

    private fun lexPrivateValue() {
        var offset = tokenStart + 1
        while (offset < endOffset && buffer[offset].isIdentifierPart()) offset++
        tokenType = RustyHandlebarsTokens.PRIVATE_VALUE
        tokenEnd = offset
    }

    private fun lexWord() {
        var offset = tokenStart + 1
        while (offset < endOffset &&
            (buffer[offset].isIdentifierPart() || buffer[offset] == '.')
        ) {
            offset++
        }
        val word = buffer.subSequence(tokenStart, offset).toString()
        tokenType = when (word) {
            "if", "unless", "if_some", "if_some_ref",
            "with", "with_ref", "each", "each_ref" ->
                RustyHandlebarsTokens.BUILTIN_BLOCK
            "lookup", "try_lookup", "format" ->
                RustyHandlebarsTokens.BUILTIN_HELPER
            "as", "else" -> RustyHandlebarsTokens.KEYWORD
            "true", "false" -> RustyHandlebarsTokens.BOOLEAN
            else -> RustyHandlebarsTokens.IDENTIFIER
        }
        tokenEnd = offset
    }

    private fun close(type: IElementType, length: Int) {
        tokenType = type
        tokenEnd = tokenStart + length
        state = DATA_STATE
    }

    private fun consumeWhile(type: IElementType, predicate: (Char) -> Boolean) {
        var offset = tokenStart + 1
        while (offset < endOffset && predicate(buffer[offset])) offset++
        tokenType = type
        tokenEnd = offset
    }

    private fun consumeTrim(offset: Int) =
        if (offset < endOffset && buffer[offset] == '~') offset + 1 else offset

    private fun endOf(needle: String, from: Int): Int {
        val found = buffer.indexOf(needle, from, endOffset)
        return if (found < 0) endOffset else found + needle.length
    }

    private fun endOfEither(first: String, second: String, from: Int): Int {
        val firstOffset = buffer.indexOf(first, from, endOffset)
        val secondOffset = buffer.indexOf(second, from, endOffset)
        return when {
            firstOffset < 0 && secondOffset < 0 -> endOffset
            firstOffset < 0 -> secondOffset + second.length
            secondOffset < 0 -> firstOffset + first.length
            firstOffset < secondOffset -> firstOffset + first.length
            else -> secondOffset + second.length
        }
    }

    private fun nextOpening(from: Int): Int {
        var searchFrom = from
        while (searchFrom < endOffset) {
            val opening = buffer.indexOf("{{", searchFrom, endOffset)
            if (opening < 0) return -1
            var backslashes = 0
            var offset = opening - 1
            while (offset >= from && buffer[offset] == '\\') {
                backslashes++
                offset--
            }
            if (backslashes % 2 == 0) return opening
            searchFrom = opening + 2
        }
        return -1
    }

    private fun startsWith(value: String) =
        tokenStart + value.length <= endOffset &&
            buffer.regionMatches(tokenStart, value, 0, value.length)

    private fun Char.isIdentifierStart() = this == '_' || isLetter()
    private fun Char.isIdentifierPart() = this == '_' || isLetterOrDigit()

    companion object {
        private const val DATA_STATE = 0
        private const val EXPRESSION_STATE = 1
    }
}

private fun CharSequence.indexOf(
    needle: String,
    startIndex: Int,
    endIndex: Int,
): Int {
    if (needle.isEmpty()) return startIndex
    val last = endIndex - needle.length
    for (offset in startIndex..last) {
        if (regionMatches(offset, needle, 0, needle.length)) return offset
    }
    return -1
}

private fun CharSequence.regionMatches(
    thisOffset: Int,
    other: String,
    otherOffset: Int,
    length: Int,
): Boolean {
    if (thisOffset < 0 || thisOffset + length > this.length) return false
    for (index in 0 until length) {
        if (this[thisOffset + index] != other[otherOffset + index]) return false
    }
    return true
}
