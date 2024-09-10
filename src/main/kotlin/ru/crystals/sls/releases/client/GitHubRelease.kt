package ru.crystals.sls.releases.client

import com.fasterxml.jackson.annotation.JsonProperty

data class GitHubRelease(@JsonProperty("tag_name") val tagName: String,
                         @JsonProperty("html_url") val url: String)
