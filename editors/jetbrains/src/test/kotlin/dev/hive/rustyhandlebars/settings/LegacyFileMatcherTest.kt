package dev.hive.rustyhandlebars.settings

import kotlin.test.Test
import kotlin.test.assertFalse
import kotlin.test.assertFailsWith
import kotlin.test.assertTrue

class LegacyFileMatcherTest {
    @Test
    fun matchesProjectRelativeGlobs() {
        assertTrue(LegacyFileMatcher.matches("templates/**/*.hbs", "templates/email.hbs"))
        assertTrue(LegacyFileMatcher.matches("templates/**/*.hbs", "templates/mail/email.hbs"))
        assertTrue(LegacyFileMatcher.matches("**/legacy-?.hbs", "views/legacy-a.hbs"))
        assertFalse(LegacyFileMatcher.matches("templates/**/*.hbs", "other/email.hbs"))
        assertFalse(LegacyFileMatcher.matches("templates/**/*.hbs", "templates/email.rhbs"))
    }

    @Test
    fun rejectsAbsoluteAndUnsupportedGlobs() {
        assertFailsWith<IllegalArgumentException> {
            LegacyFileMatcher.compile("/templates/*.hbs")
        }
        assertFailsWith<IllegalArgumentException> {
            LegacyFileMatcher.compile("templates/{a,b}.hbs")
        }
    }
}
