package dev.hive.rustyhandlebars.settings

object LegacyFileMatcher {
    fun matches(pattern: String, projectRelativePath: String): Boolean =
        compile(pattern).matches(projectRelativePath.replace('\\', '/'))

    fun compile(pattern: String): Regex {
        val normalized = pattern.trim().replace('\\', '/')
        require(normalized.isNotEmpty()) { "Legacy template globs cannot be blank." }
        require(!normalized.startsWith('/')) {
            "Legacy template globs must be relative to the project."
        }

        val expression = buildString {
            append('^')
            var index = 0
            while (index < normalized.length) {
                when (val character = normalized[index]) {
                    '*' -> {
                        if (index + 1 < normalized.length && normalized[index + 1] == '*') {
                            index++
                            if (index + 1 < normalized.length && normalized[index + 1] == '/') {
                                index++
                                append("(?:.*/)?")
                            } else {
                                append(".*")
                            }
                        } else {
                            append("[^/]*")
                        }
                    }
                    '?' -> append("[^/]")
                    '[', ']', '{', '}' -> throw IllegalArgumentException(
                        "Unsupported glob syntax in '$pattern'. Use *, **, and ?.",
                    )
                    '.', '(', ')', '+', '|', '^', '$', '\\' -> {
                        append('\\')
                        append(character)
                    }
                    else -> append(character)
                }
                index++
            }
            append('$')
        }
        return Regex(expression)
    }
}
