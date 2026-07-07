import Foundation
import Security

enum SSHSecretKind: String {
    case password
    case privateKey
    case passphrase
}

struct SSHSecretStore {
    static let shared = SSHSecretStore()

    private let service = "xyz.zinglix.shellow.ssh"

    func hasSecret(for profile: HostProfile, kind: SSHSecretKind) -> Bool {
        loadSecret(for: profile, kind: kind) != nil
    }

    func loadSecret(for profile: HostProfile, kind: SSHSecretKind) -> String? {
        var query = baseQuery(for: profile, kind: kind)
        query[kSecReturnData as String] = true
        query[kSecMatchLimit as String] = kSecMatchLimitOne

        var result: CFTypeRef?
        let status = SecItemCopyMatching(query as CFDictionary, &result)
        guard
            status == errSecSuccess,
            let data = result as? Data,
            let secret = String(data: data, encoding: .utf8),
            !secret.isEmpty
        else {
            return nil
        }
        return secret
    }

    func saveSecret(_ secret: String, for profile: HostProfile, kind: SSHSecretKind) throws {
        let data = Data(secret.utf8)
        var query = baseQuery(for: profile, kind: kind)

        let attributes: [String: Any] = [
            kSecValueData as String: data,
            kSecAttrAccessible as String: kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly
        ]

        let updateStatus = SecItemUpdate(query as CFDictionary, attributes as CFDictionary)
        if updateStatus == errSecSuccess {
            return
        }

        guard updateStatus == errSecItemNotFound else {
            throw SSHSecretStoreError.keychainStatus(updateStatus)
        }

        query.merge(attributes) { _, new in new }
        let addStatus = SecItemAdd(query as CFDictionary, nil)
        guard addStatus == errSecSuccess else {
            throw SSHSecretStoreError.keychainStatus(addStatus)
        }
    }

    func deleteSecret(for profile: HostProfile, kind: SSHSecretKind) {
        SecItemDelete(baseQuery(for: profile, kind: kind) as CFDictionary)
    }

    private func baseQuery(for profile: HostProfile, kind: SSHSecretKind) -> [String: Any] {
        [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: "\(profile.id.uuidString).\(kind.rawValue)"
        ]
    }
}

enum SSHSecretStoreError: Error {
    case keychainStatus(OSStatus)
}
