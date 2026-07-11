import Foundation

struct HostProfile: Identifiable, Hashable, Codable {
    var id = UUID()
    var name: String
    var host: String
    var port: Int
    var username: String
    var authentication: AuthenticationKind
    var launchKind: ProfileLaunchKind? = nil
    var trustedHostKeySHA256: String?
    // Keep the v1 JSON key for backward compatibility. New code reads and
    // writes this through `persistentTerminal` so existing tmux hosts migrate
    // to the generic multiplexer model without losing their saved session.
    var tmuxSession: PersistentTerminalConfiguration? = nil
    var capabilityReport: RemoteHostCapabilityReport? = nil
    var lastConnected: Date?

    var endpoint: String {
        "\(username)@\(host):\(port)"
    }

    var resolvedLaunchKind: ProfileLaunchKind {
        launchKind ?? .terminal
    }

    var hostKeyTrustTitle: String {
        if let trustedHostKeySHA256, !trustedHostKeySHA256.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            "Host key pinned"
        } else {
            "Host key unverified"
        }
    }

    var persistentTerminal: PersistentTerminalConfiguration? {
        get {
            guard
                var configuration = tmuxSession,
                let name = PersistentTerminalConfiguration.validatedName(configuration.name)
            else {
                return nil
            }
            configuration.name = name
            return configuration
        }
        set {
            tmuxSession = newValue
        }
    }

    var terminalStartupCommand: String {
        guard let configuration = persistentTerminal else { return "" }
        let backend = configuration.backend
        return "if command -v \(backend.executable) >/dev/null 2>&1; then \(backend.attachCommand(sessionName: configuration.name)); else echo 'Shellow: \(backend.displayTitle) is not installed; continuing with the regular shell.'; fi"
    }
}

enum ProfileLaunchKind: String, CaseIterable, Identifiable, Codable {
    case terminal
    case codex
    case claude

    var id: String { rawValue }

    var title: String {
        switch self {
        case .terminal: "Terminal"
        case .codex: "Codex"
        case .claude: "Claude Code"
        }
    }

    var systemImage: String {
        switch self {
        case .terminal: "terminal"
        case .codex: "sparkles"
        case .claude: "bolt.horizontal.circle"
        }
    }

    var detail: String {
        switch self {
        case .terminal: "Open a remote shell and persistent workspaces"
        case .codex: "Open remote coding conversations"
        case .claude: "Open durable Claude Code sessions over SSH"
        }
    }
}

enum PersistentTerminalBackend: String, CaseIterable, Identifiable, Codable {
    case tmux
    case screen
    case zellij

    var id: String { rawValue }

    var displayTitle: String {
        switch self {
        case .tmux: "tmux"
        case .screen: "GNU screen"
        case .zellij: "Zellij"
        }
    }

    var compactTitle: String {
        switch self {
        case .tmux: "tmux"
        case .screen: "screen"
        case .zellij: "Zellij"
        }
    }

    var executable: String { rawValue }

    var controlPrefix: String {
        switch self {
        case .tmux: "\u{2}"
        case .screen: "\u{1}"
        case .zellij: "\u{F}"
        }
    }

    var controlPrefixLabel: String {
        switch self {
        case .tmux: "Ctrl-B"
        case .screen: "Ctrl-A"
        case .zellij: "Ctrl-O"
        }
    }

    var persistenceDetail: String {
        switch self {
        case .tmux:
            "Creates or attaches with tmux new-session -A."
        case .screen:
            "Attaches here with screen -D -R, creating the named session when needed."
        case .zellij:
            "Attaches to the named Zellij session or creates it when needed."
        }
    }

    func attachCommand(sessionName: String) -> String {
        switch self {
        case .tmux:
            return "tmux new-session -A -s \(sessionName)"
        case .screen:
            let exactSession = "screen_id=\"$(screen -ls 2>/dev/null | awk '$1 ~ /[.]\(sessionName)$/ { print $1; exit }')\""
            return "\(exactSession); if [ -n \"$screen_id\" ]; then screen -D -R \"$screen_id\"; else screen -S \(sessionName); fi"
        case .zellij:
            return "zellij attach --create \(sessionName)"
        }
    }
}

struct PersistentTerminalConfiguration: Hashable, Codable {
    static let maximumNameLength = 48

    var name: String
    var backend: PersistentTerminalBackend

    init(name: String, backend: PersistentTerminalBackend = .tmux) {
        self.name = name
        self.backend = backend
    }

    private enum CodingKeys: String, CodingKey {
        case name
        case backend
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        name = try container.decode(String.self, forKey: .name)
        backend = try container.decodeIfPresent(PersistentTerminalBackend.self, forKey: .backend) ?? .tmux
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode(name, forKey: .name)
        try container.encode(backend, forKey: .backend)
    }

    static func validatedName(_ value: String) -> String? {
        let trimmed = value.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty, trimmed.count <= maximumNameLength else { return nil }

        let scalars = Array(trimmed.unicodeScalars)
        guard let first = scalars.first, isASCIIAlphaNumeric(first) else { return nil }
        guard scalars.allSatisfy({ scalar in
            isASCIIAlphaNumeric(scalar) || scalar.value == 45 || scalar.value == 95
        }) else {
            return nil
        }
        return trimmed
    }

    static func suggestedName(profileName: String, host: String) -> String {
        let source = profileName.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
            ? host
            : profileName
        var slug = ""
        var lastWasSeparator = false

        for scalar in source.lowercased().unicodeScalars {
            if isASCIIAlphaNumeric(scalar) {
                slug.unicodeScalars.append(scalar)
                lastWasSeparator = false
            } else if !slug.isEmpty, !lastWasSeparator {
                slug.append("-")
                lastWasSeparator = true
            }
        }

        slug = slug.trimmingCharacters(in: CharacterSet(charactersIn: "-"))
        let candidate = "shellow-" + (slug.isEmpty ? "session" : slug)
        return String(candidate.prefix(maximumNameLength))
    }

    private static func isASCIIAlphaNumeric(_ scalar: UnicodeScalar) -> Bool {
        (48...57).contains(scalar.value)
            || (65...90).contains(scalar.value)
            || (97...122).contains(scalar.value)
    }
}

struct RemoteTerminalSessionSummary: Identifiable, Hashable {
    var name: String
    var isAttached: Bool
    var windowCount: Int?

    var id: String { name }
}

struct RemoteTerminalSessionCatalog: Equatable {
    var sessions: [RemoteTerminalSessionSummary]
    var errorMessage: String?

    static let empty = Self(sessions: [], errorMessage: nil)
}

enum RemoteTerminalSessionProbe {
    private static let marker = "__SHELLOW_SESSIONS_V1__"

    static func command(for backend: PersistentTerminalBackend) -> String {
        let body: String
        switch backend {
        case .tmux:
            body = """
            tmux list-sessions -F 'session|#{session_name}|#{session_attached}|#{session_windows}' 2>/dev/null || true
            """
        case .screen:
            body = """
            screen -ls 2>/dev/null | awk '
              /^[[:space:]]*[0-9]+[.]/ {
                name=$1; sub(/^[0-9]+[.]/, "", name);
                attached=(index($0, "(Attached)") > 0 ? 1 : 0);
                printf "session|%s|%d|\\n", name, attached;
              }
            ' || true
            """
        case .zellij:
            body = """
            zellij list-sessions --no-formatting 2>/dev/null | awk '
              NF {
                name=$1;
                if (name ~ /^[A-Za-z0-9][A-Za-z0-9_-]*$/) {
                  attached=(index(tolower($0), "current") > 0 || index(tolower($0), "attached") > 0 ? 1 : 0);
                  printf "session|%s|%d|\\n", name, attached;
                }
              }
            ' || true
            """
        }

        return """
        LC_ALL=C
        PATH="$PATH:/opt/homebrew/bin:/usr/local/bin:/home/linuxbrew/.linuxbrew/bin:$HOME/.local/bin:$HOME/bin"
        export PATH
        printf '__SHELLOW_SESSIONS_V1__\n'
        if command -v \(backend.executable) >/dev/null 2>&1; then
        \(body)
        else
          printf 'error|\(backend.displayTitle) is not installed on this host.\n'
        fi
        """
    }

    static func parse(_ output: String) -> RemoteTerminalSessionCatalog? {
        let lines = output
            .replacingOccurrences(of: "\r", with: "")
            .split(separator: "\n", omittingEmptySubsequences: false)
            .map(String.init)
        guard lines.contains(marker) else { return nil }

        var sessions: [RemoteTerminalSessionSummary] = []
        var errorMessage: String?
        for line in lines {
            let fields = line.split(separator: "|", omittingEmptySubsequences: false).map(String.init)
            if fields.first == "session", fields.count >= 4,
               let name = PersistentTerminalConfiguration.validatedName(fields[1]) {
                let attached = Int(fields[2]).map { $0 > 0 } ?? false
                let windowCount = Int(fields[3])
                if !sessions.contains(where: { $0.name == name }) {
                    sessions.append(RemoteTerminalSessionSummary(
                        name: name,
                        isAttached: attached,
                        windowCount: windowCount
                    ))
                }
            } else if fields.first == "error", fields.count >= 2 {
                errorMessage = fields.dropFirst().joined(separator: "|")
            }
        }

        return RemoteTerminalSessionCatalog(
            sessions: sessions.sorted { $0.name.localizedStandardCompare($1.name) == .orderedAscending },
            errorMessage: errorMessage
        )
    }
}

enum RemoteComponentSupportLevel: String, Hashable, Codable {
    case supported
    case limited
    case unavailable

    var title: String {
        switch self {
        case .supported: "Full"
        case .limited: "Limited"
        case .unavailable: "Not installed"
        }
    }
}

struct RemoteSystemCapability: Hashable, Codable {
    var kernelName: String
    var operatingSystemName: String
    var operatingSystemVersion: String
    var kernelRelease: String
    var architecture: String
    var loginShell: String

    var familyTitle: String {
        switch kernelName.lowercased() {
        case "darwin": "macOS"
        case "linux": "Linux"
        case "freebsd": "FreeBSD"
        case "openbsd": "OpenBSD"
        case "netbsd": "NetBSD"
        default: kernelName.isEmpty ? "Unknown system" : kernelName
        }
    }

    var displayTitle: String {
        let name = operatingSystemName.isEmpty ? familyTitle : operatingSystemName
        return operatingSystemVersion.isEmpty ? name : "\(name) \(operatingSystemVersion)"
    }

    var shellName: String {
        let component = URL(fileURLWithPath: loginShell).lastPathComponent
        return component.isEmpty ? loginShell : component
    }
}

struct RemoteComponentCapability: Identifiable, Hashable, Codable {
    var backend: PersistentTerminalBackend
    var supportLevel: RemoteComponentSupportLevel
    var version: String

    var id: PersistentTerminalBackend { backend }

    var featureSummary: String {
        switch (backend, supportLevel) {
        case (_, .unavailable):
            "Install \(backend.displayTitle) on the target host to enable it."
        case (.tmux, .supported):
            "Attach/create, sessions, windows, pane splits, and detach are supported."
        case (.screen, .supported):
            "Exact-name attach/create, windows, horizontal regions, and detach are supported."
        case (.zellij, .supported):
            "Attach/create, sessions, tabs, pane splits, detach, and layout recovery are supported."
        case (.tmux, .limited):
            "tmux is installed, but new-session -A was not advertised; automatic restore may be unavailable."
        case (.screen, .limited):
            "GNU screen is installed, but -R attach/create support was not advertised."
        case (.zellij, .limited):
            "Zellij is installed, but attach --create support was not advertised."
        }
    }
}

struct RemoteHostCapabilityReport: Hashable, Codable {
    static let refreshInterval: TimeInterval = 24 * 60 * 60

    var detectedAt: Date
    var system: RemoteSystemCapability
    var components: [RemoteComponentCapability]

    var isStale: Bool {
        Date().timeIntervalSince(detectedAt) > Self.refreshInterval
    }

    func capability(for backend: PersistentTerminalBackend) -> RemoteComponentCapability? {
        components.first { $0.backend == backend }
    }
}

struct RemoteHostProbeOutcome {
    var report: RemoteHostCapabilityReport?
    var errorMessage: String?

    static func success(_ report: RemoteHostCapabilityReport) -> Self {
        Self(report: report, errorMessage: nil)
    }

    static func failure(_ message: String) -> Self {
        Self(report: nil, errorMessage: message)
    }
}

enum RemoteHostCapabilityProbe {
    private static let marker = "__SHELLOW_CAPABILITIES_V1__"

    static let command = """
    LC_ALL=C
    PATH="$PATH:/opt/homebrew/bin:/usr/local/bin:/home/linuxbrew/.linuxbrew/bin:$HOME/.local/bin:$HOME/bin"
    export PATH
    one_line() { printf '%s' "$1" | tr '|\r\n' '   '; }
    kernel_name="$(uname -s 2>/dev/null || printf unknown)"
    kernel_release="$(uname -r 2>/dev/null || printf unknown)"
    architecture="$(uname -m 2>/dev/null || printf unknown)"
    login_shell="${SHELL:-unknown}"
    os_name="$kernel_name"
    os_version=""
    if [ "$kernel_name" = Darwin ] && command -v sw_vers >/dev/null 2>&1; then
      os_name="$(sw_vers -productName 2>/dev/null || printf macOS)"
      os_version="$(sw_vers -productVersion 2>/dev/null || true)"
    elif [ -r /etc/os-release ]; then
      . /etc/os-release
      os_name="${NAME:-$kernel_name}"
      os_version="${VERSION_ID:-}"
    fi
    printf '__SHELLOW_CAPABILITIES_V1__\n'
    printf 'system|%s|%s|%s|%s|%s|%s\n' "$(one_line "$kernel_name")" "$(one_line "$os_name")" "$(one_line "$os_version")" "$(one_line "$kernel_release")" "$(one_line "$architecture")" "$(one_line "$login_shell")"
    if command -v tmux >/dev/null 2>&1; then
      version="$(tmux -V 2>&1 | head -n 1)"
      if tmux list-commands 2>/dev/null | grep '^new-session ' | grep -q 'A'; then level=supported; else level=limited; fi
      printf 'component|tmux|%s|%s\n' "$level" "$(one_line "$version")"
    else
      printf 'component|tmux|unavailable|\n'
    fi
    if command -v screen >/dev/null 2>&1; then
      version="$(screen --version 2>&1 | head -n 1)"
      if screen -help 2>&1 | grep -q -- '-R'; then level=supported; else level=limited; fi
      printf 'component|screen|%s|%s\n' "$level" "$(one_line "$version")"
    else
      printf 'component|screen|unavailable|\n'
    fi
    if command -v zellij >/dev/null 2>&1; then
      version="$(zellij --version 2>&1 | head -n 1)"
      if zellij attach --help 2>&1 | grep -q -- '--create'; then level=supported; else level=limited; fi
      printf 'component|zellij|%s|%s\n' "$level" "$(one_line "$version")"
    else
      printf 'component|zellij|unavailable|\n'
    fi
    """

    static func parse(_ output: String, detectedAt: Date = .now) -> RemoteHostCapabilityReport? {
        let lines = output
            .replacingOccurrences(of: "\r", with: "")
            .split(separator: "\n", omittingEmptySubsequences: false)
            .map(String.init)

        guard lines.contains(marker) else { return nil }

        var system: RemoteSystemCapability?
        var components: [RemoteComponentCapability] = []

        for line in lines {
            let fields = line.split(separator: "|", omittingEmptySubsequences: false).map(String.init)
            if fields.first == "system", fields.count >= 7 {
                system = RemoteSystemCapability(
                    kernelName: fields[1],
                    operatingSystemName: fields[2],
                    operatingSystemVersion: fields[3],
                    kernelRelease: fields[4],
                    architecture: fields[5],
                    loginShell: fields[6]
                )
            } else if fields.first == "component",
                      fields.count >= 4,
                      let backend = PersistentTerminalBackend(rawValue: fields[1]),
                      let supportLevel = RemoteComponentSupportLevel(rawValue: fields[2]) {
                components.append(RemoteComponentCapability(
                    backend: backend,
                    supportLevel: supportLevel,
                    version: fields[3]
                ))
            }
        }

        guard let system else { return nil }
        let completedComponents = PersistentTerminalBackend.allCases.map { backend in
            components.first { $0.backend == backend }
                ?? RemoteComponentCapability(
                    backend: backend,
                    supportLevel: .unavailable,
                    version: ""
                )
        }
        return RemoteHostCapabilityReport(
            detectedAt: detectedAt,
            system: system,
            components: completedComponents
        )
    }
}

struct SSHKeyCredential: Identifiable, Hashable, Codable {
    var id = UUID()
    var name: String
    var createdAt = Date()
}

enum AuthenticationKind: String, CaseIterable, Identifiable, Codable {
    case password
    case privateKey

    var id: String { rawValue }

    var title: String {
        switch self {
        case .password: "Password"
        case .privateKey: "Private Key"
        }
    }
}

extension HostProfile {
    static let samples: [HostProfile] = [
        HostProfile(
            name: "Staging",
            host: "staging.example.com",
            port: 22,
            username: "deploy",
            authentication: .privateKey,
            launchKind: .terminal,
            trustedHostKeySHA256: "SHA256:sample-staging-host-key",
            lastConnected: .now.addingTimeInterval(-1_800)
        ),
        HostProfile(
            name: "Workshop",
            host: "shell.example.com",
            port: 22,
            username: "ops",
            authentication: .password,
            launchKind: .codex,
            trustedHostKeySHA256: nil,
            lastConnected: .now.addingTimeInterval(-86_400)
        )
    ]
}

enum HostProfileStore {
    private static let key = "shellow.hostProfiles.v1"

    static func load() -> [HostProfile] {
        guard let data = UserDefaults.standard.data(forKey: key) else {
            return HostProfile.samples
        }

        do {
            let profiles = try JSONDecoder().decode([HostProfile].self, from: data)
            return profiles.isEmpty ? HostProfile.samples : profiles
        } catch {
            return HostProfile.samples
        }
    }

    static func save(_ profiles: [HostProfile]) {
        guard let data = try? JSONEncoder().encode(profiles) else {
            return
        }
        UserDefaults.standard.set(data, forKey: key)
    }
}

enum SSHKeyCredentialStore {
    private static let key = "shellow.sshKeyCredentials.v1"

    static func load() -> [SSHKeyCredential] {
        guard let data = UserDefaults.standard.data(forKey: key) else {
            return []
        }

        do {
            return try JSONDecoder().decode([SSHKeyCredential].self, from: data)
        } catch {
            return []
        }
    }

    static func save(_ credentials: [SSHKeyCredential]) {
        guard let data = try? JSONEncoder().encode(credentials) else {
            return
        }
        UserDefaults.standard.set(data, forKey: key)
    }
}
