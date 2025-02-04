package ru.crystals.sls.releases.client.github

import com.fasterxml.jackson.annotation.JsonProperty

internal data class GitHubRelease(@JsonProperty("tag_name") val tagName: String,
                                  @JsonProperty("html_url") val url: String)
