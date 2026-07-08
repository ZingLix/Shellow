import SwiftUI

struct HostsScreen: View {
    @Binding var profiles: [HostProfile]
    let onOpenSettings: () -> Void
    let connectPreview: (HostProfile) -> Void
    let connectPassword: (HostProfile, String, String) -> Void
    let connectPrivateKey: (HostProfile, String, String?, String) -> Void

    @State private var draftName = ""
    @State private var draftHost = ""
    @State private var draftPort = "22"
    @State private var draftUser = ""
    @State private var draftHostKeyFingerprint = ""
    @State private var authentication: AuthenticationKind = .privateKey
    @State private var selectedPasswordProfile: HostProfile?
    @State private var selectedPrivateKeyProfile: HostProfile?
    @State private var livePassword = ""
    @State private var livePrivateKey = ""
    @State private var liveKeyPassphrase = ""
    @State private var liveCommand = ""
    @State private var isAddingProfile = false

    var body: some View {
        List {
            Section("Hosts") {
                ForEach(profiles) { profile in
                    Button {
                        open(profile)
                    } label: {
                        HostProfileRow(profile: profile)
                    }
                    .buttonStyle(.plain)
                }
            }
        }
        .navigationTitle("Shellow")
        .toolbar {
            ToolbarItem(placement: .topBarLeading) {
                Button(action: onOpenSettings) {
                    Image(systemName: "gearshape")
                }
                .accessibilityLabel("Settings")
            }

            ToolbarItem(placement: .topBarTrailing) {
                Button {
                    isAddingProfile = true
                } label: {
                    Image(systemName: "plus")
                }
                .accessibilityLabel("Add Host")
            }
        }
        .sheet(isPresented: $isAddingProfile) {
            NewHostProfileSheet(
                draftName: $draftName,
                draftHost: $draftHost,
                draftPort: $draftPort,
                draftUser: $draftUser,
                draftHostKeyFingerprint: $draftHostKeyFingerprint,
                authentication: $authentication,
                addProfile: addProfile
            )
            .presentationDetents([.large])
        }
        .sheet(item: $selectedPasswordProfile) { profile in
            LiveSSHPasswordSheet(
                profile: profile,
                password: $livePassword,
                command: $liveCommand,
                connect: { password, command in
                    connectPassword(profile, password, command)
                    selectedPasswordProfile = nil
                }
            )
            .presentationDetents([.medium])
        }
        .sheet(item: $selectedPrivateKeyProfile) { profile in
            LiveSSHPrivateKeySheet(
                profile: profile,
                privateKey: $livePrivateKey,
                passphrase: $liveKeyPassphrase,
                command: $liveCommand,
                preview: {
                    connectPreview(profile)
                    selectedPrivateKeyProfile = nil
                },
                connect: { privateKeyPEM, passphrase, command in
                    connectPrivateKey(
                        profile,
                        privateKeyPEM,
                        passphrase,
                        command
                    )
                    selectedPrivateKeyProfile = nil
                }
            )
            .presentationDetents([.large])
        }
    }

    private func open(_ profile: HostProfile) {
        if profile.authentication == .password {
            livePassword = ""
            liveCommand = ""
            selectedPasswordProfile = profile
        } else {
            livePrivateKey = ""
            liveKeyPassphrase = ""
            liveCommand = ""
            selectedPrivateKeyProfile = profile
        }
    }

    private var canAddProfile: Bool {
        !draftName.isEmpty && !draftHost.isEmpty && !draftUser.isEmpty && Int(draftPort) != nil
    }

    private func addProfile() {
        guard canAddProfile, let port = Int(draftPort) else {
            return
        }

        profiles.append(
            HostProfile(
                name: draftName,
                host: draftHost,
                port: port,
                username: draftUser,
                authentication: authentication,
                trustedHostKeySHA256: normalizedDraftHostKeyFingerprint,
                lastConnected: nil
            )
        )

        draftName = ""
        draftHost = ""
        draftPort = "22"
        draftUser = ""
        draftHostKeyFingerprint = ""
        authentication = .privateKey
    }

    private var normalizedDraftHostKeyFingerprint: String? {
        let value = draftHostKeyFingerprint.trimmingCharacters(in: .whitespacesAndNewlines)
        return value.isEmpty ? nil : value
    }

}

private struct NewHostProfileSheet: View {
    @Binding var draftName: String
    @Binding var draftHost: String
    @Binding var draftPort: String
    @Binding var draftUser: String
    @Binding var draftHostKeyFingerprint: String
    @Binding var authentication: AuthenticationKind
    let addProfile: () -> Void

    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            Form {
                Section("Connection") {
                    TextField("Name", text: $draftName)
                    TextField("Host", text: $draftHost)
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()
                    TextField("Port", text: $draftPort)
                        .keyboardType(.numberPad)
                    TextField("User", text: $draftUser)
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()
                }

                Section("Authentication") {
                    Picker("Auth", selection: $authentication) {
                        ForEach(AuthenticationKind.allCases) { kind in
                            Text(kind.title).tag(kind)
                        }
                    }

                    TextField("Host key SHA256", text: $draftHostKeyFingerprint)
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()
                        .font(.system(.body, design: .monospaced))
                }
            }
            .navigationTitle("New Host")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") {
                        dismiss()
                    }
                }

                ToolbarItem(placement: .confirmationAction) {
                    Button("Add") {
                        addProfile()
                        dismiss()
                    }
                    .disabled(!canAddProfile)
                }
            }
        }
    }

    private var canAddProfile: Bool {
        !draftName.isEmpty && !draftHost.isEmpty && !draftUser.isEmpty && Int(draftPort) != nil
    }
}

private struct LiveSSHPasswordSheet: View {
    let profile: HostProfile
    @Binding var password: String
    @Binding var command: String
    let connect: (String, String) -> Void
    @Environment(\.dismiss) private var dismiss
    @State private var rememberPassword = false
    @State private var hasSavedPassword = false
    @State private var keychainStatus: String?

    private let secretStore = SSHSecretStore.shared

    var body: some View {
        NavigationStack {
            Form {
                Section("Host") {
                    LabeledContent("Name", value: profile.name)
                    LabeledContent("Endpoint", value: profile.endpoint)
                    LabeledContent("Host Key", value: profile.hostKeyTrustTitle)
                }

                Section("Interactive Shell") {
                    if hasSavedPassword {
                        Label("Saved password available", systemImage: "key.fill")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                    }
                    SecureField("Password", text: $password)
                        .textContentType(.password)
                    Toggle("Save password in Keychain", isOn: $rememberPassword)
                    TextField("Startup command", text: $command)
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()
                        .font(.system(.body, design: .monospaced))
                    if let keychainStatus {
                        Text(keychainStatus)
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                    }
                }
            }
            .navigationTitle("Live SSH")
            .navigationBarTitleDisplayMode(.inline)
            .onAppear {
                refreshSavedState()
                rememberPassword = false
            }
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") {
                        dismiss()
                    }
                }

                ToolbarItem(placement: .confirmationAction) {
                    Button("Connect") {
                        connectWithResolvedPassword()
                    }
                    .disabled(!canConnect)
                }
            }
        }
    }

    private var canConnect: Bool {
        !password.isEmpty || hasSavedPassword
    }

    private func refreshSavedState() {
        hasSavedPassword = secretStore.hasSecret(for: profile, kind: .password)
    }

    private func connectWithResolvedPassword() {
        let resolvedPassword: String
        if password.isEmpty {
            guard let saved = secretStore.loadSecret(for: profile, kind: .password) else {
                keychainStatus = "Saved password could not be loaded"
                refreshSavedState()
                return
            }
            resolvedPassword = saved
        } else {
            resolvedPassword = password
        }

        if rememberPassword && !password.isEmpty {
            do {
                try secretStore.saveSecret(password, for: profile, kind: .password)
                keychainStatus = "Password saved in Keychain"
                refreshSavedState()
            } catch {
                keychainStatus = "Keychain save failed"
            }
        }

        connect(resolvedPassword, command)
        dismiss()
    }
}

private struct LiveSSHPrivateKeySheet: View {
    let profile: HostProfile
    @Binding var privateKey: String
    @Binding var passphrase: String
    @Binding var command: String
    let preview: () -> Void
    let connect: (String, String?, String) -> Void
    @Environment(\.dismiss) private var dismiss
    @State private var rememberPrivateKey = false
    @State private var rememberPassphrase = false
    @State private var hasSavedPrivateKey = false
    @State private var hasSavedPassphrase = false
    @State private var keychainStatus: String?

    private let secretStore = SSHSecretStore.shared

    var body: some View {
        NavigationStack {
            Form {
                Section("Host") {
                    LabeledContent("Name", value: profile.name)
                    LabeledContent("Endpoint", value: profile.endpoint)
                    LabeledContent("Host Key", value: profile.hostKeyTrustTitle)

                    Button {
                        preview()
                    } label: {
                        Label("Preview Terminal", systemImage: "terminal")
                    }
                }

                Section("Private Key") {
                    if hasSavedPrivateKey {
                        Label("Saved private key available", systemImage: "key.fill")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                    }
                    TextEditor(text: $privateKey)
                        .font(.system(.footnote, design: .monospaced))
                        .frame(minHeight: 180)
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()
                    Toggle("Save private key in Keychain", isOn: $rememberPrivateKey)
                    if hasSavedPassphrase {
                        Label("Saved passphrase available", systemImage: "lock.fill")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                    }
                    SecureField("Passphrase", text: $passphrase)
                        .textContentType(.password)
                    Toggle("Save passphrase in Keychain", isOn: $rememberPassphrase)
                    TextField("Startup command", text: $command)
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()
                        .font(.system(.body, design: .monospaced))
                    if let keychainStatus {
                        Text(keychainStatus)
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                    }
                }
            }
            .navigationTitle("Live SSH Key")
            .navigationBarTitleDisplayMode(.inline)
            .onAppear {
                refreshSavedState()
                rememberPrivateKey = !hasSavedPrivateKey
                rememberPassphrase = false
            }
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") {
                        dismiss()
                    }
                }

                ToolbarItem(placement: .confirmationAction) {
                    Button("Connect") {
                        connectWithResolvedKey()
                    }
                    .disabled(!canConnect)
                }
            }
        }
    }

    private var canConnect: Bool {
        privateKeyLooksUsable(privateKey) || hasSavedPrivateKey
    }

    private func refreshSavedState() {
        hasSavedPrivateKey = secretStore.hasSecret(for: profile, kind: .privateKey)
        hasSavedPassphrase = secretStore.hasSecret(for: profile, kind: .passphrase)
    }

    private func connectWithResolvedKey() {
        let resolvedPrivateKey: String
        if privateKey.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            guard let savedKey = secretStore.loadSecret(for: profile, kind: .privateKey) else {
                keychainStatus = "Saved private key could not be loaded"
                refreshSavedState()
                return
            }
            resolvedPrivateKey = savedKey
        } else {
            resolvedPrivateKey = privateKey
        }

        guard privateKeyLooksUsable(resolvedPrivateKey) else {
            keychainStatus = "Private key is not an OpenSSH key"
            return
        }

        if rememberPrivateKey && privateKeyLooksUsable(privateKey) {
            do {
                try secretStore.saveSecret(privateKey, for: profile, kind: .privateKey)
                keychainStatus = "Private key saved in Keychain"
                refreshSavedState()
            } catch {
                keychainStatus = "Private key save failed"
            }
        }

        if rememberPassphrase && !passphrase.isEmpty {
            do {
                try secretStore.saveSecret(passphrase, for: profile, kind: .passphrase)
                refreshSavedState()
            } catch {
                keychainStatus = "Passphrase save failed"
            }
        }

        let resolvedPassphrase =
            passphrase.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
                ? secretStore.loadSecret(for: profile, kind: .passphrase)
                : passphrase

        connect(resolvedPrivateKey, resolvedPassphrase, command)
        dismiss()
    }
}

private func privateKeyLooksUsable(_ value: String) -> Bool {
    value.contains("BEGIN") && value.contains("PRIVATE KEY")
}

private struct HostProfileRow: View {
    let profile: HostProfile

    var body: some View {
        HStack(spacing: 12) {
            Image(systemName: "server.rack")
                .font(.system(size: 17, weight: .semibold))
                .frame(width: 34, height: 34)
                .foregroundStyle(ShellowTheme.accent)
                .background(ShellowTheme.accent.opacity(0.14), in: RoundedRectangle(cornerRadius: 8))

            VStack(alignment: .leading, spacing: 3) {
                Text(profile.name)
                    .font(.body.weight(.semibold))
                Text(profile.endpoint)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                Text(profile.hostKeyTrustTitle)
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }

            Spacer()

            Text(profile.authentication.title)
                .font(.caption.weight(.medium))
                .foregroundStyle(.secondary)

            Image(systemName: "chevron.right")
                .font(.caption.weight(.semibold))
                .foregroundStyle(.tertiary)
        }
        .padding(.vertical, 4)
    }
}

#Preview {
    HostsScreen(
        profiles: .constant(HostProfile.samples),
        onOpenSettings: {},
        connectPreview: { _ in },
        connectPassword: { _, _, _ in },
        connectPrivateKey: { _, _, _, _ in }
    )
}
