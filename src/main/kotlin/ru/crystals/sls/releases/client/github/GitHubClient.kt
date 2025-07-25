package ru.crystals.sls.releases.client.github

import com.fasterxml.jackson.databind.DeserializationFeature
import io.ktor.client.*
import io.ktor.client.call.*
import io.ktor.client.engine.cio.*
import io.ktor.client.plugins.contentnegotiation.*
import io.ktor.client.plugins.logging.*
import io.ktor.client.request.*
import io.ktor.http.*
import io.ktor.serialization.jackson.*
import ru.crystals.sls.releases.model.release.Release
import java.util.concurrent.ConcurrentHashMap
import java.time.Instant
import java.time.Duration
import com.github.benmanes.caffeine.cache.Caffeine
import com.github.benmanes.caffeine.cache.Cache

class GitHubClient(val token: String) {
    // Data class for cached page
    private data class CachedPage(val value: Collection<GitHubRelease>, val expiresAt: Instant)
    // Caffeine cache for pages, with manual expiration
    private val pageCache: Cache<Int, CachedPage> =
        Caffeine.newBuilder().build()

    private val client = HttpClient(CIO) {
        install(ContentNegotiation) {
            jackson(
                block = {
                    disable(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES)
                }
            )
        }

        install(Logging) {
            logger = Logger.DEFAULT
            level = LogLevel.HEADERS
        }
    }

    suspend fun getReleases(converter: Converter): Collection<Release> {
        var page = 0
        var list: Collection<GitHubRelease>
        val result = ArrayList<Release>()
        do {
            list = getPage(page++)
            list.stream()
                .mapMulti(converter::convert)
                .forEach { r -> result.add(r) }
        } while (list.isNotEmpty())

        return result
    }

    private suspend fun getPage(page: Int) : Collection<GitHubRelease> {
        val now = Instant.now()
        // Check cache
        pageCache.getIfPresent(page)?.let { cached ->
            if (now.isBefore(cached.expiresAt)) {
                return cached.value
            } else {
                pageCache.invalidate(page)
            }
        }

        val url = "https://api.github.com/repos/crystalservice/SET10-Loyalty/releases?per_page=100&page=$page"

        val response = client.get(url) {
            headers {
                append(HttpHeaders.Accept, "application/vnd.github+json")
                append(HttpHeaders.Authorization, "Bearer $token")
                append("X-GitHub-Api-Version", "2022-11-28")
            }
        }

        println("Got http status " + response.status.value)

        if (response.status.value != 200)
            throw RuntimeException("Got unexpected response $response")

        val list: Collection<GitHubRelease> = response.body()

        // Determine cache expiration
        val cacheControl = response.headers[HttpHeaders.CacheControl]
        val maxAge = cacheControl?.let {
            val match = Regex("max-age=(\\d+)").find(it)
            match?.groupValues?.getOrNull(1)?.toLongOrNull()
        }
        val expiresAt = now.plusSeconds(maxAge ?: 60L)
        pageCache.put(page, CachedPage(list, expiresAt))

        return list
    }

}