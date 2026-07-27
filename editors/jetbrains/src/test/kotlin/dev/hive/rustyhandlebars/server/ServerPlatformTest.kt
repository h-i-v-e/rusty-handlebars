package dev.hive.rustyhandlebars.server

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class ServerPlatformTest {
    @Test
    fun mapsSupportedPlatforms() {
        assertEquals(ServerPlatform.DARWIN_ARM64, ServerPlatform.detect("Mac OS X", "aarch64"))
        assertEquals(ServerPlatform.DARWIN_X64, ServerPlatform.detect("Darwin", "x86_64"))
        assertEquals(ServerPlatform.LINUX_ARM64, ServerPlatform.detect("Linux", "arm64"))
        assertEquals(ServerPlatform.LINUX_X64, ServerPlatform.detect("Linux", "amd64"))
        assertEquals(ServerPlatform.WINDOWS_X64, ServerPlatform.detect("Windows 11", "x64"))
    }

    @Test
    fun rejectsUnsupportedPlatforms() {
        assertNull(ServerPlatform.detect("Windows 11", "arm64"))
        assertNull(ServerPlatform.detect("FreeBSD", "x86_64"))
    }
}
