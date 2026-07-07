import Foundation

struct HostProfile: Identifiable, Hashable, Codable {
    var id = UUID()
    var name: String
    var host: String
    var port: Int
    var username: String
    var authentication: AuthenticationKind
    var trustedHostKeySHA256: String?
    var lastConnected: Date?

    var endpoint: String {
        "\(username)@\(host):\(port)"
    }

    var hostKeyTrustTitle: String {
        if let trustedHostKeySHA256, !trustedHostKeySHA256.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            "Host key pinned"
        } else {
            "Host key unverified"
        }
    }
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
            host: "10.0.0.18",
            port: 22,
            username: "deploy",
            authentication: .privateKey,
            trustedHostKeySHA256: "SHA256:sample-staging-host-key",
            lastConnected: .now.addingTimeInterval(-1_800)
        ),
        HostProfile(
            name: "Home Lab",
            host: "192.168.1.42",
            port: 22,
            username: "zinglix",
            authentication: .password,
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
