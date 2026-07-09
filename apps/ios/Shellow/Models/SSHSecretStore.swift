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

    func hasSecret(forKeyID keyID: UUID, kind: SSHSecretKind) -> Bool {
        loadSecret(forKeyID: keyID, kind: kind) != nil
    }

    func loadSecret(for profile: HostProfile, kind: SSHSecretKind) -> String? {
        var query = baseQuery(account: profileAccount(for: profile, kind: kind))
        query[kSecReturnData as String] = true
        query[kSecMatchLimit as String] = kSecMatchLimitOne

        return loadSecret(query: query)
    }

    func loadSecret(forKeyID keyID: UUID, kind: SSHSecretKind) -> String? {
        var query = baseQuery(account: keyAccount(for: keyID, kind: kind))
        query[kSecReturnData as String] = true
        query[kSecMatchLimit as String] = kSecMatchLimitOne

        return loadSecret(query: query)
    }

    private func loadSecret(query: [String: Any]) -> String? {
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
        try saveSecret(secret, account: profileAccount(for: profile, kind: kind))
    }

    func saveSecret(_ secret: String, forKeyID keyID: UUID, kind: SSHSecretKind) throws {
        try saveSecret(secret, account: keyAccount(for: keyID, kind: kind))
    }

    private func saveSecret(_ secret: String, account: String) throws {
        let data = Data(secret.utf8)
        var query = baseQuery(account: account)

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
        SecItemDelete(baseQuery(account: profileAccount(for: profile, kind: kind)) as CFDictionary)
    }

    func deleteSecret(forKeyID keyID: UUID, kind: SSHSecretKind) {
        SecItemDelete(baseQuery(account: keyAccount(for: keyID, kind: kind)) as CFDictionary)
    }

    private func baseQuery(account: String) -> [String: Any] {
        [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: account
        ]
    }

    private func profileAccount(for profile: HostProfile, kind: SSHSecretKind) -> String {
        "\(profile.id.uuidString).\(kind.rawValue)"
    }

    private func keyAccount(for keyID: UUID, kind: SSHSecretKind) -> String {
        "key.\(keyID.uuidString).\(kind.rawValue)"
    }
}

enum SSHSecretStoreError: Error {
    case keychainStatus(OSStatus)
}
