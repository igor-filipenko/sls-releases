package ru.crystals.sls.releases.model.release

data class ModuleRelease(val version: Version, val url: String, val dateTime: String) {

    fun asCsvRow(): String = "$version, $dateTime, $url"

    fun asHtmlRow(): String = """
        <tr>
          <td><a href='$url'>$version</a></td>
          <td>$dateTime</td>
        </tr>
    """.trimIndent()

}

