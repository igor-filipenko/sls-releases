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

class GitHubClient {
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

    suspend fun getReleases(): Collection<GitHubRelease> {
        val response = client.get("https://api.github.com/repos/crystalservice/SET10-Loyalty/releases?per_page=100") {
            parameters {
                append("per_page", "100")
                append("page", "2")
            }
            headers {
                append(HttpHeaders.Accept, "application/vnd.github+json")
                append(HttpHeaders.Authorization, "Bearer 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")
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