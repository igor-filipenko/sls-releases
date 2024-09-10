package ru.crystals.sls.releases.plugins

import ru.crystals.sls.releases.client.GitHubRelease
import java.util.function.Consumer
import kotlin.test.*

class ParserTest {

    val knownModules = hashMapOf(
        Pair("foo", "bar")
    )

    val parser = Parser(knownModules)

    @Test
    fun parse() {
        val ghr = GitHubRelease("foo-v1.2.3", "http://some/url")
        val consumer = TestConsumer()

        parser.parse(ghr, consumer)

        assertNotNull(consumer.result)
        val release: Release = consumer.result!!
        assertEquals("foo", release.name)
        assertEquals("bar", release.localizedName)
        assertEquals("http://some/url", release.url)
        assertIs<Version.Release>(release.version)
        val version = release.version as Version.Release
        assertEquals(1, version.major)
        assertEquals(2, version.minor)
        assertEquals(3, version.patch)
    }

    @Test
    fun parseCandidate() {
        val ghr = GitHubRelease("foo-v1.2.3-RC6", "http://some/url")
        val consumer = TestConsumer()

        parser.parse(ghr, consumer)

        assertNotNull(consumer.result)
        val release: Release = consumer.result!!
        assertEquals("foo", release.name)
        assertEquals("bar", release.localizedName)
        assertEquals("http://some/url", release.url)
        assertIs<Version.Candidate>(release.version)
        val version = release.version as Version.Candidate
        assertEquals(1, version.major)
        assertEquals(2, version.minor)
        assertEquals(3, version.patch)
        assertEquals(6, version.number)
    }

    @Test
    fun parseInvalid() {
        val ghr = GitHubRelease("foo-v1.2.3-SNAPSHOT", "http://some/url")
        val consumer = TestConsumer()

        parser.parse(ghr, consumer)

        assertNull(consumer.result)
    }

    class TestConsumer() : Consumer<Release> {
        var result: Release? = null

        override fun accept(t: Release) {
            result = t
        }
    }

}