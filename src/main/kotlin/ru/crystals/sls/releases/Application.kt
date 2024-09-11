package ru.crystals.sls.releases

import io.ktor.server.application.*
import ru.crystals.sls.releases.client.GitHubClient
import ru.crystals.sls.releases.plugins.Parser
import ru.crystals.sls.releases.plugins.configureRouting

fun main(args: Array<String>) {
    io.ktor.server.netty.EngineMain.main(args)
}

fun Application.module() {
    val token = environment.config.property("github.token").getString()
    val knownModules= environment.config.config("sls.modules").toMap()
        .mapValues { it.value.toString() }
    configureRouting(GitHubClient(token), Parser(knownModules))
}
