package ru.crystals.sls.releases.client

import com.fasterxml.jackson.databind.DeserializationFeature
import io.ktor.client.*
import io.ktor.client.call.*
import io.ktor.client.engine.cio.*
import io.ktor.client.plugins.contentnegotiation.*
import io.ktor.client.plugins.logging.*
import io.ktor.client.request.*
import io.ktor.http.*
import io.ktor.serialization.jackson.*
import ru.crystals.sls.releases.plugins.Parser
import ru.crystals.sls.releases.plugins.Release
import ru.crystals.sls.releases.plugins.Version

class GitHubClient(val token: String) {
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

    suspend fun getReleases(parser: Parser): Collection<Release> {
        var page = 0
        var list: Collection<GitHubRelease>
        val result = ArrayList<Release>()
        do {
            list = getPage(page++)
            list.stream()
                .mapMulti(parser::parse)
                .filter { r -> r.version is Version.Release }
                .forEach { r -> result.add(r) }
        } while (list.isNotEmpty())

        return result
    }

    private suspend fun getPage(page: Int) : Collection<GitHubRelease> {
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

        return list
    }

}