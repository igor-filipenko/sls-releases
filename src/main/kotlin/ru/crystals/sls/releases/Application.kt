package ru.crystals.sls.releases

import io.ktor.server.application.*
import ru.crystals.sls.releases.client.GitHubClient
import ru.crystals.sls.releases.plugins.configureRouting

fun main(args: Array<String>) {
    io.ktor.server.netty.EngineMain.main(args)
}

fun Application.module() {
    val token = environment.config.property("github.token").getString()
    configureRouting(GitHubClient(token))
}
