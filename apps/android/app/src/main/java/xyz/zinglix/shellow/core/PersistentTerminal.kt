package xyz.zinglix.shellow.core

import java.util.Locale
import org.json.JSONArray
import org.json.JSONObject

enum class PersistentTerminalBackend(
  val wire: String,
  val displayTitle: String,
  val compactTitle: String,
  val controlPrefixLabel: String,
) {
  Tmux("tmux", "tmux", "tmux", "Ctrl-B"),
  Screen("screen", "GNU screen", "screen", "Ctrl-A"),
  Zellij("zellij", "Zellij", "Zellij", "Ctrl-O");

  val executable: String = wire

  val detachSequence: String
    get() =
      when (this) {
        Tmux -> "\u0002d"
        Screen -> "\u0001d"
        Zellij -> "\u000Fd"
      }

  val persistenceDetail: String
    get() =
      when (this) {
        Tmux -> "Attaches to the named tmux session or creates it when needed."
        Screen -> "Uses an exact screen session name and reattaches it here."
        Zellij -> "Attaches to the named Zellij session or creates it when needed."
      }

  fun attachCommand(sessionName: String): String =
    when (this) {
      Tmux -> "tmux new-session -A -s $sessionName"
      Screen -> {
        val exactSession = "screen_id=\"\$(screen -ls 2>/dev/null | awk '\$1 ~ /[.]${sessionName}\$/ { print \$1; exit }')\""
        "$exactSession; if [ -n \"\$screen_id\" ]; then screen -D -R \"\$screen_id\"; else screen -S $sessionName; fi"
      }
      Zellij -> "zellij attach --create $sessionName"
    }

  companion object {
    fun fromWire(value: String?): PersistentTerminalBackend? = entries.firstOrNull { it.wire == value }
  }
}

data class PersistentTerminalConfiguration(
  val name: String,
  val backend: PersistentTerminalBackend = PersistentTerminalBackend.Tmux,
) {
  fun toJson(): JSONObject =
    JSONObject()
      .put("name", name)
      .put("backend", backend.wire)

  companion object {
    const val MaximumNameLength = 48

    fun fromJson(json: JSONObject?): PersistentTerminalConfiguration? {
      if (json == null) return null
      val name = validatedName(json.optString("name")) ?: return null
      val backend = PersistentTerminalBackend.fromWire(json.optString("backend")) ?: PersistentTerminalBackend.Tmux
      return PersistentTerminalConfiguration(name, backend)
    }

    fun validatedName(value: String): String? {
      val trimmed = value.trim()
      if (trimmed.isEmpty() || trimmed.length > MaximumNameLength) return null
      if (!trimmed.first().isAsciiAlphaNumeric()) return null
      if (!trimmed.all { it.isAsciiAlphaNumeric() || it == '-' || it == '_' }) return null
      return trimmed
    }

    fun suggestedName(profileName: String, host: String): String {
      val source = profileName.trim().ifEmpty { host.trim() }.lowercase(Locale.ROOT)
      val slug =
        buildString {
          var lastWasSeparator = false
          source.forEach { character ->
            if (character.isAsciiAlphaNumeric()) {
              append(character)
              lastWasSeparator = false
            } else if (isNotEmpty() && !lastWasSeparator) {
              append('-')
              lastWasSeparator = true
            }
          }
        }.trim('-')
      return "shellow-${slug.ifEmpty { "session" }}".take(MaximumNameLength)
    }
  }
}

data class RemoteTerminalSessionSummary(
  val name: String,
  val isAttached: Boolean,
  val windowCount: Int?,
)

data class RemoteTerminalSessionCatalog(
  val sessions: List<RemoteTerminalSessionSummary> = emptyList(),
  val errorMessage: String? = null,
)

object RemoteTerminalSessionProbe {
  private const val Marker = "__SHELLOW_SESSIONS_V1__"

  fun command(backend: PersistentTerminalBackend): String {
    val body =
      when (backend) {
        PersistentTerminalBackend.Tmux ->
          """
          tmux list-sessions -F 'session|#{session_name}|#{session_attached}|#{session_windows}' 2>/dev/null || true
          """.trimIndent()
        PersistentTerminalBackend.Screen ->
          """
          screen -ls 2>/dev/null | awk '
            /^[[:space:]]*[0-9]+[.]/ {
              name=${'$'}1; sub(/^[0-9]+[.]/, "", name);
              attached=(index(${'$'}0, "(Attached)") > 0 ? 1 : 0);
              printf "session|%s|%d|\n", name, attached;
            }
          ' || true
          """.trimIndent()
        PersistentTerminalBackend.Zellij ->
          """
          zellij list-sessions --no-formatting 2>/dev/null | awk '
            NF {
              name=${'$'}1;
              if (name ~ /^[A-Za-z0-9][A-Za-z0-9_-]*${'$'}/) {
                attached=(index(tolower(${'$'}0), "current") > 0 || index(tolower(${'$'}0), "attached") > 0 ? 1 : 0);
                printf "session|%s|%d|\n", name, attached;
              }
            }
          ' || true
          """.trimIndent()
      }

    return """
      LC_ALL=C
      PATH="${'$'}PATH:/opt/homebrew/bin:/usr/local/bin:/home/linuxbrew/.linuxbrew/bin:${'$'}HOME/.local/bin:${'$'}HOME/bin"
      export PATH
      printf '$Marker\n'
      if command -v ${backend.executable} >/dev/null 2>&1; then
      $body
      else
        printf 'error|${backend.displayTitle} is not installed on this host.\n'
      fi
    """.trimIndent()
  }

  fun parse(output: String): RemoteTerminalSessionCatalog? {
    val lines = output.replace("\r", "").split('\n')
    if (Marker !in lines) return null

    val sessions = linkedMapOf<String, RemoteTerminalSessionSummary>()
    var errorMessage: String? = null
    lines.forEach { line ->
      val fields = line.split('|')
      when {
        fields.firstOrNull() == "session" && fields.size >= 4 -> {
          val name = PersistentTerminalConfiguration.validatedName(fields[1]) ?: return@forEach
          sessions.putIfAbsent(
            name,
            RemoteTerminalSessionSummary(
              name = name,
              isAttached = fields[2].toIntOrNull()?.let { it > 0 } ?: false,
              windowCount = fields[3].toIntOrNull(),
            ),
          )
        }
        fields.firstOrNull() == "error" && fields.size >= 2 -> {
          errorMessage = fields.drop(1).joinToString("|")
        }
      }
    }

    return RemoteTerminalSessionCatalog(
      sessions = sessions.values.sortedBy { it.name.lowercase(Locale.ROOT) },
      errorMessage = errorMessage,
    )
  }
}

private fun Char.isAsciiAlphaNumeric(): Boolean = this in 'a'..'z' || this in 'A'..'Z' || this in '0'..'9'

enum class RemoteComponentSupportLevel(val wire: String, val title: String) {
  Supported("supported", "Full"),
  Limited("limited", "Limited"),
  Unavailable("unavailable", "Not installed");

  companion object {
    fun fromWire(value: String?): RemoteComponentSupportLevel? = entries.firstOrNull { it.wire == value }
  }
}

data class RemoteSystemCapability(
  val kernelName: String,
  val operatingSystemName: String,
  val operatingSystemVersion: String,
  val kernelRelease: String,
  val architecture: String,
  val loginShell: String,
) {
  val familyTitle: String
    get() =
      when (kernelName.lowercase(Locale.ROOT)) {
        "darwin" -> "macOS"
        "linux" -> "Linux"
        "freebsd" -> "FreeBSD"
        "openbsd" -> "OpenBSD"
        "netbsd" -> "NetBSD"
        else -> kernelName.ifEmpty { "Unknown system" }
      }

  val displayTitle: String
    get() {
      val name = operatingSystemName.ifEmpty { familyTitle }
      return if (operatingSystemVersion.isEmpty()) name else "$name $operatingSystemVersion"
    }

  val shellName: String
    get() = loginShell.substringAfterLast('/').ifEmpty { loginShell }

  fun toJson(): JSONObject =
    JSONObject()
      .put("kernelName", kernelName)
      .put("operatingSystemName", operatingSystemName)
      .put("operatingSystemVersion", operatingSystemVersion)
      .put("kernelRelease", kernelRelease)
      .put("architecture", architecture)
      .put("loginShell", loginShell)

  companion object {
    fun fromJson(json: JSONObject): RemoteSystemCapability =
      RemoteSystemCapability(
        kernelName = json.optString("kernelName"),
        operatingSystemName = json.optString("operatingSystemName"),
        operatingSystemVersion = json.optString("operatingSystemVersion"),
        kernelRelease = json.optString("kernelRelease"),
        architecture = json.optString("architecture"),
        loginShell = json.optString("loginShell"),
      )
  }
}

data class RemoteComponentCapability(
  val backend: PersistentTerminalBackend,
  val supportLevel: RemoteComponentSupportLevel,
  val version: String,
) {
  val featureSummary: String
    get() =
      when {
        supportLevel == RemoteComponentSupportLevel.Unavailable ->
          "Install ${backend.displayTitle} on the target host to enable it."
        supportLevel == RemoteComponentSupportLevel.Limited ->
          "${backend.displayTitle} is installed, but automatic attach/create was not advertised."
        backend == PersistentTerminalBackend.Tmux ->
          "Sessions, windows, pane splits, switching, and detach are supported."
        backend == PersistentTerminalBackend.Screen ->
          "Exact-name restore, windows, horizontal regions, switching, and detach are supported."
        else ->
          "Sessions, tabs, pane splits, switching, detach, and layout recovery are supported."
      }

  fun toJson(): JSONObject =
    JSONObject()
      .put("backend", backend.wire)
      .put("supportLevel", supportLevel.wire)
      .put("version", version)

  companion object {
    fun fromJson(json: JSONObject): RemoteComponentCapability? {
      val backend = PersistentTerminalBackend.fromWire(json.optString("backend")) ?: return null
      val level = RemoteComponentSupportLevel.fromWire(json.optString("supportLevel")) ?: return null
      return RemoteComponentCapability(backend, level, json.optString("version"))
    }
  }
}

data class RemoteHostCapabilityReport(
  val detectedAtEpochMillis: Long,
  val system: RemoteSystemCapability,
  val components: List<RemoteComponentCapability>,
) {
  val isStale: Boolean
    get() = System.currentTimeMillis() - detectedAtEpochMillis > RefreshIntervalMillis

  fun capability(backend: PersistentTerminalBackend): RemoteComponentCapability? =
    components.firstOrNull { it.backend == backend }

  fun toJson(): JSONObject =
    JSONObject()
      .put("detectedAtEpochMillis", detectedAtEpochMillis)
      .put("system", system.toJson())
      .put("components", JSONArray().also { values -> components.forEach { values.put(it.toJson()) } })

  companion object {
    const val RefreshIntervalMillis = 24L * 60L * 60L * 1000L

    fun fromJson(json: JSONObject?): RemoteHostCapabilityReport? {
      if (json == null) return null
      val systemJson = json.optJSONObject("system") ?: return null
      val values = json.optJSONArray("components") ?: JSONArray()
      val components =
        List(values.length()) { index -> RemoteComponentCapability.fromJson(values.getJSONObject(index)) }
          .filterNotNull()
      return RemoteHostCapabilityReport(
        detectedAtEpochMillis = json.optLong("detectedAtEpochMillis"),
        system = RemoteSystemCapability.fromJson(systemJson),
        components =
          PersistentTerminalBackend.entries.map { backend ->
            components.firstOrNull { it.backend == backend }
              ?: RemoteComponentCapability(backend, RemoteComponentSupportLevel.Unavailable, "")
          },
      )
    }
  }
}

data class RemoteHostProbeOutcome(
  val report: RemoteHostCapabilityReport? = null,
  val errorMessage: String? = null,
  val observedHostKeySha256: String? = null,
)

object RemoteHostCapabilityProbe {
  private const val Marker = "__SHELLOW_CAPABILITIES_V1__"

  val command =
    """
    LC_ALL=C
    PATH="${'$'}PATH:/opt/homebrew/bin:/usr/local/bin:/home/linuxbrew/.linuxbrew/bin:${'$'}HOME/.local/bin:${'$'}HOME/bin"
    export PATH
    one_line() { printf '%s' "${'$'}1" | tr '|\r\n' '   '; }
    kernel_name="${'$'}(uname -s 2>/dev/null || printf unknown)"
    kernel_release="${'$'}(uname -r 2>/dev/null || printf unknown)"
    architecture="${'$'}(uname -m 2>/dev/null || printf unknown)"
    login_shell="${'$'}{SHELL:-unknown}"
    os_name="${'$'}kernel_name"
    os_version=""
    if [ "${'$'}kernel_name" = Darwin ] && command -v sw_vers >/dev/null 2>&1; then
      os_name="${'$'}(sw_vers -productName 2>/dev/null || printf macOS)"
      os_version="${'$'}(sw_vers -productVersion 2>/dev/null || true)"
    elif [ -r /etc/os-release ]; then
      . /etc/os-release
      os_name="${'$'}{NAME:-${'$'}kernel_name}"
      os_version="${'$'}{VERSION_ID:-}"
    fi
    printf '$Marker\n'
    printf 'system|%s|%s|%s|%s|%s|%s\n' "${'$'}(one_line "${'$'}kernel_name")" "${'$'}(one_line "${'$'}os_name")" "${'$'}(one_line "${'$'}os_version")" "${'$'}(one_line "${'$'}kernel_release")" "${'$'}(one_line "${'$'}architecture")" "${'$'}(one_line "${'$'}login_shell")"
    if command -v tmux >/dev/null 2>&1; then
      version="${'$'}(tmux -V 2>&1 | head -n 1)"
      if tmux list-commands 2>/dev/null | grep '^new-session ' | grep -q 'A'; then level=supported; else level=limited; fi
      printf 'component|tmux|%s|%s\n' "${'$'}level" "${'$'}(one_line "${'$'}version")"
    else
      printf 'component|tmux|unavailable|\n'
    fi
    if command -v screen >/dev/null 2>&1; then
      version="${'$'}(screen --version 2>&1 | head -n 1)"
      if screen -help 2>&1 | grep -q -- '-R'; then level=supported; else level=limited; fi
      printf 'component|screen|%s|%s\n' "${'$'}level" "${'$'}(one_line "${'$'}version")"
    else
      printf 'component|screen|unavailable|\n'
    fi
    if command -v zellij >/dev/null 2>&1; then
      version="${'$'}(zellij --version 2>&1 | head -n 1)"
      if zellij attach --help 2>&1 | grep -q -- '--create'; then level=supported; else level=limited; fi
      printf 'component|zellij|%s|%s\n' "${'$'}level" "${'$'}(one_line "${'$'}version")"
    else
      printf 'component|zellij|unavailable|\n'
    fi
    """.trimIndent()

  fun parse(output: String, detectedAtEpochMillis: Long = System.currentTimeMillis()): RemoteHostCapabilityReport? {
    val lines = output.replace("\r", "").split('\n')
    if (Marker !in lines) return null

    var system: RemoteSystemCapability? = null
    val components = mutableListOf<RemoteComponentCapability>()
    lines.forEach { line ->
      val fields = line.split('|')
      when {
        fields.firstOrNull() == "system" && fields.size >= 7 -> {
          system =
            RemoteSystemCapability(
              kernelName = fields[1],
              operatingSystemName = fields[2],
              operatingSystemVersion = fields[3],
              kernelRelease = fields[4],
              architecture = fields[5],
              loginShell = fields[6],
            )
        }
        fields.firstOrNull() == "component" && fields.size >= 4 -> {
          val backend = PersistentTerminalBackend.fromWire(fields[1])
          val level = RemoteComponentSupportLevel.fromWire(fields[2])
          if (backend != null && level != null) {
            components += RemoteComponentCapability(backend, level, fields[3])
          }
        }
      }
    }

    val detectedSystem = system ?: return null
    return RemoteHostCapabilityReport(
      detectedAtEpochMillis = detectedAtEpochMillis,
      system = detectedSystem,
      components =
        PersistentTerminalBackend.entries.map { backend ->
          components.firstOrNull { it.backend == backend }
            ?: RemoteComponentCapability(backend, RemoteComponentSupportLevel.Unavailable, "")
        },
    )
  }

  fun outcome(session: TerminalSession): RemoteHostProbeOutcome {
    val output = session.rows.joinToString("\n") { it.text }
    val report = parse(output)
    if (report != null) {
      return RemoteHostProbeOutcome(
        report = report,
        observedHostKeySha256 = session.observedHostKeySha256,
      )
    }

    val detail =
      session.rows
        .asReversed()
        .firstOrNull { it.style == TerminalRowStyle.Muted || it.style == TerminalRowStyle.Warning }
        ?.text
        ?.takeIf { it.isNotBlank() }
        ?: "The target did not return a recognizable capability report."
    return RemoteHostProbeOutcome(errorMessage = detail, observedHostKeySha256 = session.observedHostKeySha256)
  }
}
