import SwiftUI
import UIKit

struct HostsScreen: View {
    @Binding var profiles: [HostProfile]
    @Binding var sshKeys: [SSHKeyCredential]
    let onOpenSettings: () -> Void
    let connectTerminal: (HostProfile) -> Void
    let connectCodex: (HostProfile) -> Void

    @State private var draftName = ""
    @State private var draftHost = ""
    @State private var draftPort = "22"
    @State private var draftUser = ""
    @State private var isAddingProfile = false
    @State private var isManagingKeys = false
    @State private var selectedProfile: HostProfile?

    var body: some View {
        List {
            Section("Hosts") {
                if profiles.isEmpty {
                    ContentUnavailableView {
                        Label("No Hosts", systemImage: "server.rack")
                    } description: {
                        Text("Add a host to start a Terminal or Codex session.")
                    } actions: {
                        Button("Add Host") {
                            isAddingProfile = true
                        }
                    }
                } else {
                    ForEach(profiles) { profile in
                        HostProfileRow(
                            profile: profile,
                            open: {
                                selectedProfile = profile
                            }
                        )
                    }
                }
            }
        }
        .navigationTitle("Shellow")
        .accessibilityHidden(isPresentingSheet)
        .allowsHitTesting(!isPresentingSheet)
        .toolbar {
            ToolbarItem(placement: .topBarTrailing) {
                HStack(spacing: 12) {
                    Button {
                        isAddingProfile = true
                    } label: {
                        Image(systemName: "plus")
                    }
                    .accessibilityLabel("Add Host")

                    Menu {
                        Button(action: onOpenSettings) {
                            Label("Settings", systemImage: "gearshape")
                        }

                        Button {
                            isManagingKeys = true
                        } label: {
                            Label("SSH Keys", systemImage: "key")
                        }
                    } label: {
                        Image(systemName: "ellipsis.circle")
                    }
                    .accessibilityLabel("Manage")
                }
                .accessibilityHidden(isPresentingSheet)
                .allowsHitTesting(!isPresentingSheet)
            }
        }
        .sheet(item: $selectedProfile) { profile in
            HostConnectionSheet(
                profile: profile,
                connectTerminal: {
                    selectedProfile = nil
                    connectTerminal(profile)
                },
                connectCodex: {
                    selectedProfile = nil
                    connectCodex(profile)
                }
            )
            .presentationDetents([.medium])
        }
        .sheet(isPresented: $isAddingProfile) {
            NewHostProfileSheet(
                draftName: $draftName,
                draftHost: $draftHost,
                draftPort: $draftPort,
                draftUser: $draftUser,
                addProfile: addProfile
            )
            .presentationDetents([.large])
        }
        .sheet(isPresented: $isManagingKeys) {
            SSHKeyManagementSheet(
                credentials: $sshKeys
            )
            .presentationDetents([.large])
        }
    }

    private var isPresentingSheet: Bool {
        selectedProfile != nil || isAddingProfile || isManagingKeys
    }

    private var canAddProfile: Bool {
        !draftName.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
            && !draftHost.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
            && !draftUser.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
            && validDraftPort != nil
    }

    private func addProfile() {
        guard canAddProfile, let port = validDraftPort else {
            return
        }

        profiles.append(
            HostProfile(
                name: draftName,
                host: draftHost,
                port: port,
                username: draftUser,
                authentication: .privateKey,
                trustedHostKeySHA256: nil,
                lastConnected: nil
            )
        )

        draftName = ""
        draftHost = ""
        draftPort = "22"
        draftUser = ""
    }

    private var validDraftPort: Int? {
        guard let port = Int(draftPort), (1...65535).contains(port) else {
            return nil
        }
        return port
    }

}

private struct HostConnectionSheet: View {
    let profile: HostProfile
    let connectTerminal: () -> Void
    let connectCodex: () -> Void

    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            Form {
                Section("Connection") {
                    HostConnectionSummary(
                        profile: profile,
                        reason: nil
                    )
                }

                Section("Connect") {
                    Button {
                        dismiss()
                        connectTerminal()
                    } label: {
                        ConnectionModeRow(
                            title: "Terminal",
                            subtitle: "Open an interactive shell",
                            systemImage: "terminal"
                        )
                    }
                    .buttonStyle(.plain)

                    Button {
                        dismiss()
                        connectCodex()
                    } label: {
                        ConnectionModeRow(
                            title: "Codex",
                            subtitle: "Start a remote coding session",
                            systemImage: "sparkles"
                        )
                    }
                    .buttonStyle(.plain)
                }
            }
            .navigationTitle(profile.name)
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") {
                        dismiss()
                    }
                }
            }
        }
    }
}

private struct ConnectionModeRow: View {
    let title: String
    let subtitle: String
    let systemImage: String

    var body: some View {
        HStack(spacing: 12) {
            Image(systemName: systemImage)
                .font(.system(size: 17, weight: .semibold))
                .frame(width: 28, height: 28)
                .foregroundStyle(.secondary)

            VStack(alignment: .leading, spacing: 2) {
                Text(title)
                    .font(.body.weight(.semibold))
                    .foregroundStyle(.primary)
                Text(subtitle)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            Spacer()

            Image(systemName: "chevron.right")
                .font(.caption.weight(.semibold))
                .foregroundStyle(.tertiary)
        }
        .padding(.vertical, 4)
        .contentShape(Rectangle())
    }
}

private struct NewHostProfileSheet: View {
    @Binding var draftName: String
    @Binding var draftHost: String
    @Binding var draftPort: String
    @Binding var draftUser: String
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
                    if let profileRequirement {
                        Text(profileRequirement)
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                    }
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
        profileRequirement == nil
    }

    private var profileRequirement: String? {
        if draftName.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            return "Enter a name for this host."
        }
        if draftHost.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            return "Enter a hostname or IP address."
        }
        if draftUser.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            return "Enter the SSH username."
        }
        guard let port = Int(draftPort), (1...65535).contains(port) else {
            return "Port must be a number from 1 to 65535."
        }
        return nil
    }
}

struct PasswordPromptSheet: View {
    let profile: HostProfile
    let modeTitle: String
    let reason: String?
    let connect: (String) -> Void

    @Environment(\.dismiss) private var dismiss
    @State private var password = ""
    @State private var rememberPassword = true
    @State private var keychainStatus: String?

    private let secretStore = SSHSecretStore.shared

    var body: some View {
        NavigationStack {
            Form {
                Section("Connection") {
                    HostConnectionSummary(
                        profile: profile,
                        reason: reason
                    )
                }

                Section("Password") {
                    SecureField("Password", text: $password)
                        .textContentType(.password)
                    Toggle("Save in Keychain", isOn: $rememberPassword)
                    if let keychainStatus {
                        Text(keychainStatus)
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                    } else if password.isEmpty {
                        Text("Enter a password to connect.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                    }
                }
            }
            .navigationTitle(modeTitle)
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") {
                        dismiss()
                    }
                }

                ToolbarItem(placement: .confirmationAction) {
                    Button("Connect") {
                        connectWithPassword()
                    }
                    .disabled(password.isEmpty)
                }
            }
        }
    }

    private func connectWithPassword() {
        if rememberPassword {
            do {
                try secretStore.saveSecret(password, for: profile, kind: .password)
            } catch {
                keychainStatus = "Password could not be saved"
                return
            }
        }

        connect(password)
        dismiss()
    }
}

private struct HostConnectionSummary: View {
    let profile: HostProfile
    let reason: String?

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(profile.endpoint)
                .font(.body.weight(.semibold))
            Text(profile.hostKeyTrustTitle)
                .font(.caption)
                .foregroundStyle(.secondary)
            if let reason {
                Text(reason)
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                    .padding(.top, 2)
            }
        }
        .padding(.vertical, 2)
    }
}

private struct SSHKeyManagementSheet: View {
    @Binding var credentials: [SSHKeyCredential]

    @Environment(\.dismiss) private var dismiss
    @State private var isAddingKey = false

    private let secretStore = SSHSecretStore.shared

    var body: some View {
        NavigationStack {
            List {
                Section("Keys") {
                    if credentials.isEmpty {
                        ContentUnavailableView {
                            Label("No SSH Keys", systemImage: "key")
                        } description: {
                            Text("Add a private key for key-based authentication.")
                        } actions: {
                            Button("Add Key") {
                                isAddingKey = true
                            }
                        }
                    } else {
                        ForEach(credentials) { credential in
                            HStack(spacing: 12) {
                                Image(systemName: "key")
                                    .frame(width: 28, height: 28)
                                    .foregroundStyle(ShellowTheme.accent)

                                VStack(alignment: .leading, spacing: 2) {
                                    Text(credential.name)
                                        .font(.body.weight(.semibold))
                                    Text(credential.id.uuidString)
                                        .font(.caption2.monospaced())
                                        .foregroundStyle(.secondary)
                                        .lineLimit(1)
                                }

                                Spacer()

                                Button(role: .destructive) {
                                    delete(credential)
                                } label: {
                                    Image(systemName: "trash")
                                }
                                .buttonStyle(.borderless)
                                .accessibilityLabel("Delete Key")
                            }
                        }
                    }
                }
            }
            .navigationTitle("SSH Keys")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Done") {
                        dismiss()
                    }
                }

                ToolbarItem(placement: .confirmationAction) {
                    Button {
                        isAddingKey = true
                    } label: {
                        Image(systemName: "plus")
                    }
                    .accessibilityLabel("Add Key")
                }
            }
            .sheet(isPresented: $isAddingKey) {
                AddSSHKeySheet { credential in
                    credentials.append(credential)
                    isAddingKey = false
                }
                .presentationDetents([.large])
            }
        }
    }

    private func delete(_ credential: SSHKeyCredential) {
        credentials.removeAll { $0.id == credential.id }
        secretStore.deleteSecret(forKeyID: credential.id, kind: .privateKey)
        secretStore.deleteSecret(forKeyID: credential.id, kind: .passphrase)
    }
}

private struct AddSSHKeySheet: View {
    let onAdd: (SSHKeyCredential) -> Void

    @Environment(\.dismiss) private var dismiss
    @State private var name = ""
    @State private var privateKey = ""
    @State private var passphrase = ""
    @State private var keychainStatus: String?

    private let secretStore = SSHSecretStore.shared

    var body: some View {
        NavigationStack {
            Form {
                Section("Key") {
                    TextField("Name", text: $name)
                    Text("OpenSSH Private Key")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                    TextEditor(text: $privateKey)
                        .font(.system(.footnote, design: .monospaced))
                        .frame(minHeight: 180)
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()
                    Text("Paste an OpenSSH private key.")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                    if let keyRequirement {
                        Text(keyRequirement)
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                    }
                }

                Section("Passphrase") {
                    SecureField("Optional passphrase", text: $passphrase)
                        .textContentType(.password)
                    if let keychainStatus {
                        Text(keychainStatus)
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                    }
                }
            }
            .navigationTitle("New Key")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") {
                        dismiss()
                    }
                }

                ToolbarItem(placement: .confirmationAction) {
                    Button("Add") {
                        addKey()
                    }
                    .disabled(!canAdd)
                }
            }
        }
    }

    private var canAdd: Bool {
        keyRequirement == nil
    }

    private var keyRequirement: String? {
        if name.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            return "Enter a name for this key."
        }
        if !privateKeyLooksUsable(privateKey) {
            return "Paste a valid OpenSSH private key."
        }
        return nil
    }

    private func addKey() {
        let credential = SSHKeyCredential(
            name: name.trimmingCharacters(in: .whitespacesAndNewlines)
        )

        do {
            try secretStore.saveSecret(privateKey, forKeyID: credential.id, kind: .privateKey)
            if !passphrase.isEmpty {
                try secretStore.saveSecret(passphrase, forKeyID: credential.id, kind: .passphrase)
            }
        } catch {
            secretStore.deleteSecret(forKeyID: credential.id, kind: .privateKey)
            secretStore.deleteSecret(forKeyID: credential.id, kind: .passphrase)
            keychainStatus = "Key could not be saved"
            return
        }

        onAdd(credential)
        dismiss()
    }
}

struct CodexScreen: View {
    private static let chatBottomID = "codex-chat-bottom"

    let snapshot: CodexSnapshot
    let onSendMessage: (String) -> Void
    let onUpdateSettings: (String, String, String) -> Void
    let onBrowseDirectory: (String) async -> Void
    let onListThreads: (String, String, String, Bool, Bool) async -> Void
    let onStartThread: (String) async -> Void
    let onStartThreadAndSend: (String, String) async -> Void
    let onResumeThread: (String) async -> Void
    let onReadThread: (String) async -> Void
    let onLoadMoreThreadTurns: (String, String) async -> Void
    let onRenameThread: (String, String) async -> Void
    let onArchiveThread: (String) async -> Void
    let onUnarchiveThread: (String) async -> Void
    let onDeleteThread: (String) async -> Void
    let onForkThread: (String, String) async -> Void
    let onInterruptTurn: () -> Void
    let onApprovalDecision: (String, String) -> Void
    let onDisconnect: () -> Void
    let onReconnect: (() -> Void)?

    @Environment(\.dismiss) private var dismiss
    @State private var draft = ""
    @State private var selectedPath = ""
    @State private var historySearch = ""
    @State private var homeRoute = CodexHomeRoute.overview
    @State private var draftReturnRoute = CodexHomeRoute.overview
    @State private var threadReturnRoute = CodexHomeRoute.overview
    @State private var threadReturnScope = CodexHistoryScope.allProjects
    @State private var isShowingThread = false
    @State private var didLoadProjectState = false
    @State private var historyScope = CodexHistoryScope.allProjects
    @State private var showArchivedThreads = false
    @State private var showingSettings = false
    @State private var settingsModel = ""
    @State private var settingsApprovalPolicy = ""
    @State private var settingsSandbox = ""
    @State private var renameTarget: CodexThreadSummary?
    @State private var renameText = ""
    @State private var deleteTarget: CodexThreadSummary?
    @State private var openingThreadId: String?
    @State private var isStartingDraftThread = false

    var body: some View {
        VStack(spacing: 0) {
            codexHeader
            Divider()
            operationBanner
            if isShowingThread && snapshot.threadId != nil {
                chatView
                Divider()
                composer
            } else {
                homeContent
            }
        }
        .background(Color(.systemBackground))
        .onAppear {
            if snapshot.threadId != nil {
                isShowingThread = true
            }
        }
        .task(id: snapshot.status) {
            if snapshot.status != .connected {
                didLoadProjectState = false
            }
            await loadInitialProjectStateIfNeeded()
        }
        .task(id: snapshot.endpoint) {
            settingsModel = snapshot.settings.model ?? ""
            settingsApprovalPolicy = snapshot.settings.approvalPolicy ?? ""
            settingsSandbox = snapshot.settings.sandbox ?? ""
        }
        .onChange(of: snapshot.cwd) {
            if selectedPath.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty,
               let cwd = snapshot.cwd {
                selectedPath = cwd
            }
        }
        .onChange(of: snapshot.projects.recent) {
            if selectedPath.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty,
               let firstProject = snapshot.projects.recent.first {
                selectedPath = firstProject
            }
        }
        .onChange(of: snapshot.threadId) {
            if snapshot.threadId != nil {
                isShowingThread = true
            } else {
                isShowingThread = false
            }
        }
        .onChange(of: snapshot.settings) {
            settingsModel = snapshot.settings.model ?? ""
            settingsApprovalPolicy = snapshot.settings.approvalPolicy ?? ""
            settingsSandbox = snapshot.settings.sandbox ?? ""
        }
        .onChange(of: historyScope) {
            Task { await refreshHistory() }
        }
        .sheet(isPresented: $showingSettings) {
            CodexSettingsSheet(
                model: $settingsModel,
                modelOptions: modelOptions,
                isLoadingModels: snapshot.settings.isLoadingModels,
                modelsError: snapshot.settings.modelsError,
                approvalPolicy: $settingsApprovalPolicy,
                sandbox: $settingsSandbox,
                canApply: settingsCanApply,
                apply: {
                    onUpdateSettings(
                        settingsModel.trimmingCharacters(in: .whitespacesAndNewlines),
                        settingsApprovalPolicy,
                        settingsSandbox
                    )
                }
            )
        }
        .alert("Rename Thread", isPresented: Binding(
            get: { renameTarget != nil },
            set: { if !$0 { renameTarget = nil } }
        )) {
            TextField("Name", text: $renameText)
            Button("Cancel", role: .cancel) {
                renameTarget = nil
            }
            Button("Save") {
                guard let renameTarget else { return }
                let name = renameText.trimmingCharacters(in: .whitespacesAndNewlines)
                Task { await onRenameThread(renameTarget.id, name) }
                self.renameTarget = nil
            }
        }
        .confirmationDialog("Delete this thread?", isPresented: Binding(
            get: { deleteTarget != nil },
            set: { if !$0 { deleteTarget = nil } }
        )) {
            Button("Delete", role: .destructive) {
                guard let deleteTarget else { return }
                Task { await onDeleteThread(deleteTarget.id) }
            }
            Button("Cancel", role: .cancel) {}
        } message: {
            Text(deleteTarget?.displayTitle ?? "This action cannot be undone.")
        }
    }

    @ViewBuilder
    private var operationBanner: some View {
        if let message = snapshot.operation.lastError {
            CodexInlineStatusRow(text: message, tone: .warning)
                .padding(.horizontal, 14)
        } else if let message = visibleOperationSuccess {
            CodexInlineStatusRow(text: message, tone: .success)
                .padding(.horizontal, 14)
        }
    }

    private var visibleOperationSuccess: String? {
        guard let message = snapshot.operation.lastSuccess,
              !isShowingThread || snapshot.threadId == nil else {
            return nil
        }
        return isRoutineOperationSuccess(message) ? nil : message
    }

    private func isRoutineOperationSuccess(_ message: String) -> Bool {
        message.trimmingCharacters(in: .whitespacesAndNewlines) == "Codex thread resumed."
    }

    private var codexHeader: some View {
        HStack(spacing: 10) {
            CodexBackButton(accessibilityLabel: "Back") {
                goBack()
            }

            VStack(alignment: .leading, spacing: 2) {
                Text(headerTitle)
                    .font(.headline)
                    .lineLimit(1)
                    .truncationMode(.tail)
                Text(headerSubtitle)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }

            Spacer()

            if snapshot.operation.isRunning {
                ProgressView()
                    .controlSize(.small)
                    .accessibilityLabel(snapshot.operation.label ?? "Codex operation running")
            }

            Menu {
                if let threadId = snapshot.threadId {
                    if let cursor = snapshot.threadDetail.turnsNextCursor,
                       !cursor.isEmpty {
                        Button {
                            Task { await onLoadMoreThreadTurns(threadId, cursor) }
                        } label: {
                            Label("Load More History", systemImage: "clock.arrow.circlepath")
                        }
                        .disabled(snapshot.threadDetail.isLoadingMore)
                    }

                    Button {
                        Task { await onForkThread(threadId, selectedProjectPath) }
                    } label: {
                        Label("Fork Thread", systemImage: "arrow.triangle.branch")
                    }

                    Divider()
                }

                if !isShowingThread, homeRoute == .project {
                    Button {
                        showArchivedThreads.toggle()
                        Task { await refreshHistory() }
                    } label: {
                        Label(showArchivedThreads ? "Hide Archived" : "Show Archived", systemImage: "archivebox")
                    }

                    Button {
                        Task { await refreshHistory() }
                    } label: {
                        Label("Refresh", systemImage: "arrow.clockwise")
                    }
                    .disabled(!canUseProjectActions)

                    Divider()
                }

                Button {
                    presentCodexSettings()
                } label: {
                    Label("Settings", systemImage: "slider.horizontal.3")
                }

                if let onReconnect {
                    Button(action: onReconnect) {
                        Label("Reconnect", systemImage: "arrow.clockwise")
                    }
                }

                Button(role: .destructive, action: onDisconnect) {
                    Label("Disconnect", systemImage: "power")
                }
            } label: {
                CodexOverflowMenuLabel()
            }
            .accessibilityLabel("Codex Actions")
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 10)
    }

    private var headerSubtitle: String {
        let location: String
        if !isShowingThread, homeRoute == .project, !selectedProjectPath.isEmpty {
            location = codexCompactPath(selectedProjectPath)
        } else if !isShowingThread, homeRoute == .draft, !selectedProjectPath.isEmpty {
            location = codexCompactPath(selectedProjectPath)
        } else {
            location = snapshot.cwd.map(lastPathComponent) ?? snapshot.endpoint
        }
        return "\(snapshot.status.title) · \(location)"
    }

    private var headerTitle: String {
        if isShowingThread, snapshot.threadId != nil {
            return snapshot.threadDetail.thread?.displayTitle ?? snapshot.title
        }

        switch homeRoute {
        case .overview:
            return snapshot.title
        case .project:
            return selectedProjectPath.isEmpty ? snapshot.title : lastPathComponent(selectedProjectPath)
        case .draft:
            return "New Conversation"
        }
    }

    private var chatView: some View {
        ScrollViewReader { proxy in
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 10) {
                    ForEach(snapshot.pendingApprovals) { approval in
                        CodexApprovalRow(
                            approval: approval,
                            decide: { decision in
                                onApprovalDecision(approval.requestId, decision)
                            }
                        )
                        .id("approval-\(approval.requestId)")
                    }

                    ForEach(snapshot.messages.filter(\.isVisibleInChat)) { message in
                        CodexMessageRow(message: message)
                            .id(message.id)
                    }

                    Color.clear
                        .frame(height: 1)
                        .id(Self.chatBottomID)
                }
                .padding(14)
            }
            .onAppear {
                scrollToChatBottom(proxy, animated: false)
            }
            .task(id: snapshot.threadId) {
                await Task.yield()
                scrollToChatBottom(proxy, animated: false)
            }
            .onChange(of: chatScrollSignature) {
                scrollToChatBottom(proxy, animated: true)
            }
        }
    }

    private var chatScrollSignature: Int {
        var signature = snapshot.pendingApprovals.count
        signature = signature &* 31 &+ (snapshot.turnActive ? 1 : 0)
        for message in snapshot.messages where message.isVisibleInChat {
            signature = signature &* 31 &+ message.id.count
            signature = signature &* 31 &+ message.text.count
            signature = signature &* 31 &+ (message.title?.count ?? 0)
            signature = signature &* 31 &+ (message.detail?.count ?? 0)
            signature = signature &* 31 &+ (message.transcript?.count ?? 0)
            signature = signature &* 31 &+ (message.isStreaming ? 1 : 0)
            var blockContentLength = 0
            for block in message.blocks {
                blockContentLength += markdownBlockContentLength(block)
            }
            signature = signature &* 31 &+ blockContentLength
        }
        return signature
    }

    private func markdownBlockContentLength(_ block: CodexMarkdownBlock) -> Int {
        var length = 0
        length += block.id.count
        length += block.text.count
        length += block.imageAlt?.count ?? 0
        length += markdownRunContentLength(block.runs)

        for item in block.items {
            length += item.text.count
            length += markdownRunContentLength(item.runs)
        }

        for header in block.tableHeaders {
            length += markdownTableCellContentLength(header)
        }

        for row in block.tableRows {
            for cell in row {
                length += markdownTableCellContentLength(cell)
            }
        }

        return length
    }

    private func markdownTableCellContentLength(_ cell: CodexMarkdownTableCell) -> Int {
        var length = cell.text.count
        length += markdownRunContentLength(cell.runs)
        return length
    }

    private func markdownRunContentLength(_ runs: [CodexMarkdownInlineRun]) -> Int {
        var length = 0
        for run in runs {
            length += run.text.count
        }
        return length
    }

    private func scrollToChatBottom(_ proxy: ScrollViewProxy, animated: Bool) {
        DispatchQueue.main.async {
            let action = {
                proxy.scrollTo(Self.chatBottomID, anchor: .bottom)
            }
            if animated {
                withAnimation(.easeOut(duration: 0.2), action)
            } else {
                action()
            }
        }
    }

    @ViewBuilder
    private var homeContent: some View {
        switch homeRoute {
        case .overview:
            projectHistoryView
        case .project:
            projectThreadsView
        case .draft:
            draftChatView
        }
    }

    private var projectHistoryView: some View {
        VStack(spacing: 0) {
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 18) {
                    codexHomeSearchBar
                    projectsSection
                    recentConversationsSection
                }
                .padding(14)
            }
        }
    }

    private var projectThreadsView: some View {
        VStack(spacing: 0) {
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 14) {
                    projectSearchBar
                    projectConversationsSection
                }
                .padding(14)
            }
        }
    }

    private var draftChatView: some View {
        VStack(spacing: 0) {
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 14) {
                    draftWorkspaceSection
                }
                .padding(14)
            }

            Divider()
            draftComposer
        }
    }

    private var draftWorkspaceSection: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack(alignment: .center, spacing: 10) {
                CodexInlineTextField(
                    systemImage: "folder",
                    placeholder: "Workspace path",
                    text: $selectedPath,
                    submitLabel: .go
                ) {
                    guard canUseProjectActions else { return }
                    homeRoute = .project
                    Task { await selectTypedProject() }
                }

                CodexActionIconButton(
                    systemImage: "arrow.right",
                    accessibilityLabel: "Show Workspace Conversations",
                    isEnabled: canUseProjectActions
                ) {
                    homeRoute = .project
                    Task { await selectTypedProject() }
                }
            }

            if !knownProjectPaths.isEmpty {
                VStack(alignment: .leading, spacing: 8) {
                    ForEach(knownProjectPaths, id: \.self) { path in
                        CodexDirectoryRow(
                            title: lastPathComponent(path),
                            subtitle: path,
                            systemImage: "folder"
                        ) {
                            selectedPath = path
                        }
                    }
                }
            }
        }
    }

    private var projectConversationsSection: some View {
        VStack(alignment: .leading, spacing: 10) {
            CodexSectionHeader(title: "Conversations")

            if snapshot.threads.isLoading {
                CodexInlineStatusRow(text: "Loading history", isLoading: true)
            }

            if let error = snapshot.threads.error {
                CodexInlineStatusRow(text: error, tone: .warning)
            }

            ForEach(visibleThreads) { thread in
                CodexThreadRow(
                    thread: thread,
                    archived: showArchivedThreads,
                    isOpening: openingThreadId == thread.id,
                    showsProjectContext: false,
                    resume: {
                        Task { await openThread(thread) }
                    },
                    rename: {
                        renameTarget = thread
                        renameText = thread.displayTitle
                    },
                    fork: {
                        Task { await onForkThread(thread.id, selectedProjectPath.isEmpty ? thread.cwd : selectedProjectPath) }
                    },
                    archive: {
                        Task { await onArchiveThread(thread.id) }
                    },
                    unarchive: {
                        Task { await onUnarchiveThread(thread.id) }
                    },
                    delete: {
                        deleteTarget = thread
                    }
                )
            }

            if let nextCursor = snapshot.threads.nextCursor, !nextCursor.isEmpty, homeSearchTerm.isEmpty {
                CodexLoadMoreButton(isLoading: snapshot.threads.isLoadingMore) {
                    Task { await loadMoreHistory(cursor: nextCursor) }
                }
            }

            if visibleThreads.isEmpty,
               !snapshot.threads.isLoading,
               snapshot.threads.error == nil {
                CodexEmptyState(
                    title: homeSearchTerm.isEmpty ? "No Conversations" : "No Matches",
                    detail: homeSearchTerm.isEmpty ? "Start a chat in this project when you're ready." : "Try a different search.",
                    systemImage: homeSearchTerm.isEmpty ? "bubble.left.and.text.bubble.right" : "magnifyingglass"
                )
            }
        }
    }

    private var projectsSection: some View {
        VStack(alignment: .leading, spacing: 10) {
            CodexSectionHeader(title: "Projects")

            if !visibleProjectPaths.isEmpty {
                VStack(alignment: .leading, spacing: 8) {
                    ForEach(visibleProjectPaths, id: \.self) { path in
                        CodexDirectoryRow(
                            title: lastPathComponent(path),
                            subtitle: path,
                            systemImage: "folder"
                        ) {
                            selectProject(path)
                        }
                    }
                }
            }

            if visibleProjectPaths.isEmpty,
               !snapshot.threads.isLoading {
                CodexEmptyState(
                    title: homeSearchTerm.isEmpty ? "No Projects" : "No Matches",
                    detail: homeSearchTerm.isEmpty ? "Start a chat to enter a workspace path." : "Try a different search.",
                    systemImage: homeSearchTerm.isEmpty ? "folder" : "magnifyingglass"
                )
            }
        }
    }

    private var recentConversationsSection: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack(spacing: 8) {
                CodexSectionHeader(title: "Recent Conversations", detail: historyScopeDetail)

                Spacer()

                recentConversationActionsMenu
            }

            if snapshot.threads.isLoading {
                CodexInlineStatusRow(text: "Loading history", isLoading: true)
            }

            if let error = snapshot.threads.error {
                CodexInlineStatusRow(text: error, tone: .warning)
            }

            ForEach(visibleThreads) { thread in
                CodexThreadRow(
                    thread: thread,
                    archived: showArchivedThreads,
                    isOpening: openingThreadId == thread.id,
                    resume: {
                        Task { await openThread(thread) }
                    },
                    rename: {
                        renameTarget = thread
                        renameText = thread.displayTitle
                    },
                    fork: {
                        Task { await onForkThread(thread.id, selectedProjectPath.isEmpty ? thread.cwd : selectedProjectPath) }
                    },
                    archive: {
                        Task { await onArchiveThread(thread.id) }
                    },
                    unarchive: {
                        Task { await onUnarchiveThread(thread.id) }
                    },
                    delete: {
                        deleteTarget = thread
                    }
                )
            }

            if let nextCursor = snapshot.threads.nextCursor, !nextCursor.isEmpty, homeSearchTerm.isEmpty {
                CodexLoadMoreButton(isLoading: snapshot.threads.isLoadingMore) {
                    Task { await loadMoreHistory(cursor: nextCursor) }
                }
            }

            if visibleThreads.isEmpty,
               !snapshot.threads.isLoading,
               snapshot.threads.error == nil {
                CodexEmptyState(
                    title: homeSearchTerm.isEmpty ? "No Recent Conversations" : "No Matches",
                    detail: homeSearchTerm.isEmpty ? "Start a chat from a workspace to see it here." : "Try a different search.",
                    systemImage: homeSearchTerm.isEmpty ? "clock" : "magnifyingglass"
                )
            }
        }
    }

    private var recentConversationActionsMenu: some View {
        Menu {
            Picker("Scope", selection: $historyScope) {
                Text("Current Project").tag(CodexHistoryScope.currentProject)
                Text("All Projects").tag(CodexHistoryScope.allProjects)
            }

            Button {
                showArchivedThreads.toggle()
                Task { await refreshHistory() }
            } label: {
                Label(showArchivedThreads ? "Hide Archived" : "Show Archived", systemImage: "archivebox")
            }

            Button {
                Task { await refreshHistory() }
            } label: {
                Label("Refresh", systemImage: "arrow.clockwise")
            }
            .disabled(!canUseHistoryActions)
        } label: {
            CodexOverflowMenuLabel()
        }
        .buttonStyle(.plain)
        .accessibilityLabel("Conversation Actions")
    }

    private var codexHomeSearchBar: some View {
        HStack(alignment: .center, spacing: 10) {
            CodexSearchField(
                placeholder: "Search projects or conversations",
                text: $historySearch
            ) {
                Task { await refreshHistory() }
            }

            CodexActionIconButton(
                systemImage: "square.and.pencil",
                accessibilityLabel: "New Conversation",
                isEnabled: snapshot.status == .connected
            ) {
                beginDraftChat()
            }
        }
    }

    private var projectSearchBar: some View {
        HStack(alignment: .center, spacing: 10) {
            CodexSearchField(
                placeholder: "Search this project",
                text: $historySearch
            ) {
                Task { await refreshHistory() }
            }

            CodexActionIconButton(
                systemImage: "square.and.pencil",
                accessibilityLabel: "New Conversation",
                isEnabled: canUseProjectActions
            ) {
                beginDraftChat()
            }
        }
    }

    private var composer: some View {
        VStack(spacing: 4) {
            if snapshot.turnActive {
                CodexTurnStatusRow(onStop: onInterruptTurn)
            }

            HStack(alignment: .bottom, spacing: 10) {
                CodexMessageInput(
                    text: $draft,
                    placeholder: snapshot.turnActive ? "Steer Codex" : "Message Codex",
                    isActiveTurn: snapshot.turnActive
                )

                if canSend {
                    CodexActionIconButton(
                        systemImage: snapshot.turnActive ? "arrow.turn.down.right" : "paperplane.fill",
                        accessibilityLabel: snapshot.turnActive ? "Steer Codex" : "Send",
                        isEnabled: true
                    ) {
                        sendDraft()
                    }
                }
            }
        }
        .padding(.horizontal, 12)
        .padding(.top, snapshot.turnActive ? 6 : 8)
        .padding(.bottom, 10)
        .background(.bar)
    }

    private var draftComposer: some View {
        HStack(alignment: .bottom, spacing: 10) {
            CodexMessageInput(text: $draft)

            if canSendInitialDraft || isStartingDraftThread {
                CodexActionIconButton(
                    systemImage: "paperplane.fill",
                    accessibilityLabel: "Send",
                    isEnabled: canSendInitialDraft,
                    isLoading: isStartingDraftThread
                ) {
                    Task { await sendInitialDraft() }
                }
            }
        }
        .padding(12)
        .background(.bar)
    }

    private var canSend: Bool {
        snapshot.status == .connected &&
            snapshot.threadId != nil &&
            !draft.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
    }

    private var canSendInitialDraft: Bool {
        snapshot.status == .connected &&
            !isStartingDraftThread &&
            !selectedProjectPath.isEmpty &&
            !draft.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
    }

    private var canUseProjectActions: Bool {
        snapshot.status == .connected &&
            !selectedPath.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
    }

    private var canUseHistoryActions: Bool {
        snapshot.status == .connected &&
            (historyScope == .allProjects || !selectedProjectPath.isEmpty)
    }

    private var modelOptions: [CodexModelOption] {
        var options = snapshot.settings.availableModels
        if let model = normalizeModel(snapshot.settings.model),
           !options.contains(where: { $0.id == model }) {
            options.append(CodexModelOption(id: model, name: model))
        }
        return options
    }

    private var selectedModelTitle: String {
        guard let model = normalizeModel(snapshot.settings.model) else {
            return "Default"
        }
        return modelOptions.first(where: { $0.id == model })?.name ?? model
    }

    private var settingsCanApply: Bool {
        settingsModel.trimmingCharacters(in: .whitespacesAndNewlines) != (snapshot.settings.model ?? "").trimmingCharacters(in: .whitespacesAndNewlines) ||
            settingsApprovalPolicy != (snapshot.settings.approvalPolicy ?? "") ||
            settingsSandbox != (snapshot.settings.sandbox ?? "")
    }

    private func presentCodexSettings() {
        settingsModel = snapshot.settings.model ?? ""
        settingsApprovalPolicy = snapshot.settings.approvalPolicy ?? ""
        settingsSandbox = snapshot.settings.sandbox ?? ""
        showingSettings = true
    }

    private var selectedProjectPath: String {
        selectedPath.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    private var homeSearchTerm: String {
        historySearch.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    private var historyScopeDetail: String? {
        switch historyScope {
        case .currentProject:
            "Current project"
        case .allProjects:
            nil
        }
    }

    private var visibleProjectPaths: [String] {
        knownProjectPaths.filter(matchesHomeSearch)
    }

    private var visibleThreads: [CodexThreadSummary] {
        snapshot.threads.threads.filter { thread in
            homeSearchTerm.isEmpty ||
                matchesHomeSearch(thread.displayTitle) ||
                matchesHomeSearch(thread.preview) ||
                matchesHomeSearch(thread.cwd)
        }
    }

    private var knownProjectPaths: [String] {
        mergeProjects(
            snapshot.projects.recent,
            [snapshot.projects.current, snapshot.cwd].compactMap { $0 }
        )
    }

    private func matchesHomeSearch(_ value: String) -> Bool {
        let query = homeSearchTerm
        guard !query.isEmpty else { return true }
        return value.localizedCaseInsensitiveContains(query)
    }

    private func sendDraft() {
        let message = draft.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !message.isEmpty else { return }
        draft = ""
        onSendMessage(message)
    }

    private func goBack() {
        if isShowingThread && snapshot.threadId != nil {
            returnToThreadOrigin()
            return
        }

        switch homeRoute {
        case .overview:
            dismiss()
        case .project:
            historyScope = .allProjects
            homeRoute = .overview
            Task { await refreshHistory() }
        case .draft:
            if draftReturnRoute == .project, !selectedProjectPath.isEmpty {
                historyScope = .currentProject
                homeRoute = .project
                Task { await onListThreads(selectedProjectPath, historySearch, "", showArchivedThreads, false) }
            } else {
                historyScope = .allProjects
                homeRoute = .overview
                Task { await refreshHistory() }
            }
        }
    }

    private func returnToThreadOrigin() {
        isShowingThread = false

        switch threadReturnRoute {
        case .project:
            guard !selectedProjectPath.isEmpty else {
                historyScope = .allProjects
                homeRoute = .overview
                Task { await refreshHistory() }
                return
            }
            historyScope = .currentProject
            homeRoute = .project
            Task { await onListThreads(selectedProjectPath, historySearch, "", showArchivedThreads, false) }
        case .overview:
            historyScope = threadReturnScope
            homeRoute = .overview
            Task { await refreshHistory() }
        case .draft:
            homeRoute = draftReturnRoute
        }
    }

    private func sendInitialDraft() async {
        let message = draft.trimmingCharacters(in: .whitespacesAndNewlines)
        let path = selectedProjectPath
        guard !message.isEmpty, !path.isEmpty, !isStartingDraftThread else { return }

        threadReturnRoute = draftReturnRoute
        threadReturnScope = historyScope
        isStartingDraftThread = true
        draft = ""
        await onStartThreadAndSend(path, message)
        isShowingThread = true
        isStartingDraftThread = false
    }

    private func loadInitialProjectStateIfNeeded() async {
        guard snapshot.status == .connected, snapshot.threadId == nil, !didLoadProjectState else {
            return
        }
        didLoadProjectState = true
        let path = snapshot.projects.current ?? snapshot.cwd ?? snapshot.projects.recent.first ?? selectedPath
        if !path.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            selectedPath = path
        }
        await refreshHistory()
    }

    private func selectTypedProject() async {
        let path = selectedPath.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !path.isEmpty else { return }
        homeRoute = .project
        historyScope = .currentProject
        await onListThreads(path, historySearch, "", showArchivedThreads, false)
    }

    private func refreshHistory() async {
        guard canUseHistoryActions else { return }
        await onListThreads(historyCwd, historySearch, "", showArchivedThreads, false)
    }

    private func loadMoreHistory(cursor: String) async {
        guard canUseHistoryActions else { return }
        await onListThreads(historyCwd, historySearch, cursor, showArchivedThreads, true)
    }

    private func beginDraftChat() {
        draftReturnRoute = homeRoute
        homeRoute = .draft
    }

    private func openThread(_ thread: CodexThreadSummary) async {
        print("[Shellow Codex] ui open start threadId=\(thread.id) currentThreadId=\(snapshot.threadId ?? "nil")")
        threadReturnRoute = homeRoute
        threadReturnScope = historyScope
        openingThreadId = thread.id
        await onResumeThread(thread.id)
        isShowingThread = true
        print("[Shellow Codex] ui open returned threadId=\(thread.id) currentThreadId=\(snapshot.threadId ?? "nil")")
        if openingThreadId == thread.id {
            openingThreadId = nil
        }
    }

    private var historyCwd: String {
        switch historyScope {
        case .currentProject:
            selectedProjectPath
        case .allProjects:
            ""
        }
    }

    private func selectProject(_ path: String) {
        selectedPath = path
        historyScope = .currentProject
        homeRoute = .project
        Task { await onListThreads(path, historySearch, "", showArchivedThreads, false) }
    }
}

private enum CodexHomeRoute: Hashable {
    case overview
    case project
    case draft
}

private enum CodexHistoryScope: Hashable {
    case currentProject
    case allProjects
}

private struct CodexSectionHeader: View {
    let title: String
    var detail: String?

    var body: some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(title)
                .font(.headline)
                .foregroundStyle(.primary)
            if let detail, !detail.isEmpty {
                Text(detail)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }
        }
    }
}

private struct CodexOverflowMenuLabel: View {
    var body: some View {
        Image(systemName: "ellipsis")
            .font(.system(size: 17, weight: .semibold))
            .frame(width: 30, height: 30)
            .foregroundStyle(.secondary)
            .contentShape(Circle())
    }
}

private struct CodexBackButton: View {
    let accessibilityLabel: String
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            Image(systemName: "chevron.left")
                .font(.system(size: 15, weight: .semibold))
                .frame(width: 30, height: 30)
                .foregroundStyle(.secondary)
                .contentShape(Circle())
        }
        .buttonStyle(.plain)
        .accessibilityLabel(accessibilityLabel)
    }
}

private struct CodexActionIconButton: View {
    let systemImage: String
    let accessibilityLabel: String
    let isEnabled: Bool
    var isLoading = false
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            ZStack {
                if isLoading {
                    ProgressView()
                        .controlSize(.small)
                } else {
                    Image(systemName: systemImage)
                        .font(.system(size: 15, weight: .semibold))
                }
            }
            .frame(width: 34, height: 34)
            .foregroundStyle(isEnabled ? ShellowTheme.accent : Color.secondary)
            .background(Color(.tertiarySystemFill), in: RoundedRectangle(cornerRadius: 8))
            .contentShape(RoundedRectangle(cornerRadius: 8))
            .opacity(isEnabled ? 1 : 0.45)
        }
        .buttonStyle(.plain)
        .disabled(!isEnabled || isLoading)
        .accessibilityLabel(accessibilityLabel)
    }
}

private struct CodexTurnStatusRow: View {
    let onStop: () -> Void

    var body: some View {
        HStack(spacing: 8) {
            ProgressView()
                .controlSize(.mini)
            Text("Working")
                .font(.caption2)
                .foregroundStyle(.secondary)
            Spacer()
            Button(action: onStop) {
                Text("Stop")
                    .font(.caption2.weight(.semibold))
                    .foregroundStyle(.red)
                    .padding(.vertical, 2)
            }
            .buttonStyle(.plain)
            .accessibilityLabel("Interrupt Codex Turn")
        }
        .padding(.horizontal, 4)
        .padding(.vertical, 1)
    }
}

private enum CodexInlineStatusTone {
    case neutral
    case success
    case warning
}

private struct CodexInlineStatusRow: View {
    let text: String
    var tone: CodexInlineStatusTone = .neutral
    var isLoading = false

    var body: some View {
        HStack(spacing: 8) {
            if isLoading {
                ProgressView()
                    .controlSize(.mini)
            } else if tone == .success {
                Image(systemName: "checkmark.circle")
                    .font(.caption.weight(.semibold))
            } else if tone == .warning {
                Image(systemName: "exclamationmark.triangle")
                    .font(.caption.weight(.semibold))
            }

            Text(text)
                .font(.callout)
                .lineLimit(2)
        }
        .foregroundStyle(foregroundStyle)
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.horizontal, 4)
        .padding(.vertical, 6)
    }

    private var foregroundStyle: Color {
        switch tone {
        case .neutral:
            return .secondary
        case .success:
            return .green
        case .warning:
            return .orange
        }
    }
}

private struct CodexMessageInput: View {
    @Binding var text: String
    var placeholder = "Message Codex"
    var isActiveTurn = false

    var body: some View {
        TextField(placeholder, text: $text, axis: .vertical)
            .font(.body)
            .lineLimit(1...5)
            .textInputAutocapitalization(.sentences)
            .tint(ShellowTheme.accent)
            .padding(.horizontal, 10)
            .padding(.vertical, 8)
            .background(inputBackground, in: RoundedRectangle(cornerRadius: 8))
            .overlay {
                RoundedRectangle(cornerRadius: 8)
                    .stroke(inputStroke, lineWidth: isActiveTurn ? 1 : 0)
            }
            .frame(maxWidth: .infinity)
            .accessibilityLabel(placeholder)
    }

    private var inputBackground: Color {
        isActiveTurn ? ShellowTheme.accent.opacity(0.08) : Color(.tertiarySystemFill)
    }

    private var inputStroke: Color {
        isActiveTurn ? ShellowTheme.accent.opacity(0.28) : .clear
    }
}

private struct CodexLoadMoreButton: View {
    let isLoading: Bool
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: 6) {
                if isLoading {
                    ProgressView()
                        .controlSize(.small)
                } else {
                    Image(systemName: "chevron.down")
                        .font(.caption.weight(.semibold))
                }

                Text(isLoading ? "Loading" : "Load More")
                    .font(.subheadline.weight(.semibold))
            }
            .foregroundStyle(.secondary)
            .frame(maxWidth: .infinity)
            .padding(.vertical, 8)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .disabled(isLoading)
    }
}

private struct CodexSearchField: View {
    let placeholder: String
    @Binding var text: String
    let onSubmit: () -> Void

    var body: some View {
        CodexInlineTextField(
            systemImage: "magnifyingglass",
            placeholder: placeholder,
            text: $text,
            submitLabel: .search,
            onSubmit: onSubmit
        )
    }
}

private struct CodexInlineTextField: View {
    let systemImage: String
    let placeholder: String
    @Binding var text: String
    let submitLabel: SubmitLabel
    let onSubmit: () -> Void

    var body: some View {
        HStack(spacing: 8) {
            Image(systemName: systemImage)
                .font(.system(size: 13, weight: .semibold))
                .foregroundStyle(.secondary)

            TextField(placeholder, text: $text)
                .font(.subheadline)
                .textInputAutocapitalization(.never)
                .autocorrectionDisabled()
                .submitLabel(submitLabel)
                .onSubmit(onSubmit)
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 8)
        .background(Color(.tertiarySystemFill), in: RoundedRectangle(cornerRadius: 8))
        .frame(maxWidth: .infinity)
    }
}

private struct CodexEmptyState: View {
    let title: String
    let detail: String
    let systemImage: String

    var body: some View {
        VStack(spacing: 7) {
            Image(systemName: systemImage)
                .font(.system(size: 18, weight: .semibold))
                .foregroundStyle(.secondary)

            VStack(spacing: 3) {
                Text(title)
                    .font(.subheadline.weight(.semibold))
                    .foregroundStyle(.primary)
                Text(detail)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .multilineTextAlignment(.center)
            }
        }
        .frame(maxWidth: .infinity)
        .padding(.vertical, 18)
        .padding(.horizontal, 12)
    }
}

private struct CodexDirectoryRow: View {
    let title: String
    let subtitle: String
    let systemImage: String
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: 10) {
                Image(systemName: systemImage)
                    .font(.system(size: 14, weight: .semibold))
                    .frame(width: 22, height: 22)
                    .foregroundStyle(.secondary)

                VStack(alignment: .leading, spacing: 2) {
                    Text(title)
                        .font(.body.weight(.semibold))
                        .foregroundStyle(.primary)
                    Text(codexCompactPath(subtitle))
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                        .truncationMode(.middle)
                }

                Spacer()

                Image(systemName: "chevron.right")
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(.tertiary)
            }
            .padding(.horizontal, 4)
            .padding(.vertical, 8)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .accessibilityLabel("\(title), \(subtitle)")
    }
}

private struct CodexThreadRow: View {
    let thread: CodexThreadSummary
    let archived: Bool
    let isOpening: Bool
    var showsProjectContext = true
    let resume: () -> Void
    let rename: () -> Void
    let fork: () -> Void
    let archive: () -> Void
    let unarchive: () -> Void
    let delete: () -> Void

    var body: some View {
        Button(action: resume) {
            HStack(alignment: .top, spacing: 8) {
                VStack(alignment: .leading, spacing: 4) {
                    Text(thread.displayTitle)
                        .font(.body.weight(.semibold))
                        .foregroundStyle(.primary)
                        .lineLimit(1)
                        .truncationMode(.tail)
                    Text(historyMeta)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }
                .frame(maxWidth: .infinity, alignment: .leading)

                if isOpening {
                    ProgressView()
                        .controlSize(.mini)
                        .frame(width: 30, height: 30)
                }
            }
            .padding(.horizontal, 4)
            .padding(.vertical, 8)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .disabled(isOpening)
        .contextMenu {
            actionsMenu
        }
        .swipeActions(edge: .trailing, allowsFullSwipe: false) {
            Button(role: .destructive, action: delete) {
                Label("Delete", systemImage: "trash")
            }
            Button(action: archived ? unarchive : archive) {
                Label(archived ? "Unarchive" : "Archive", systemImage: archived ? "archivebox" : "archivebox.fill")
            }
        }
    }

    @ViewBuilder
    private var actionsMenu: some View {
        Button(action: rename) {
            Label("Rename", systemImage: "pencil")
        }
        Button(action: fork) {
            Label("Fork", systemImage: "arrow.triangle.branch")
        }
        if archived {
            Button(action: unarchive) {
                Label("Unarchive", systemImage: "archivebox")
            }
        } else {
            Button(action: archive) {
                Label("Archive", systemImage: "archivebox.fill")
            }
        }
        Button(role: .destructive, action: delete) {
            Label("Delete", systemImage: "trash")
        }
    }

    private var historyMeta: String {
        let date = Date(timeIntervalSince1970: TimeInterval(thread.updatedAt))
        var parts: [String] = []
        if showsProjectContext {
            parts.append(lastPathComponent(thread.cwd))
        }
        parts.append(Self.compactDateFormatter.string(from: date))
        parts.append(thread.status)
        parts = parts.filter { !$0.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty }
        if thread.forkedFromId != nil {
            parts.append("fork")
        }
        return parts.joined(separator: " · ")
    }

    private static let compactDateFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.locale = .current
        formatter.dateFormat = "MMM d, HH:mm"
        return formatter
    }()
}

private struct CodexMessageRow: View {
    let message: CodexMessage
    @State private var isExpanded = false

    var body: some View {
        if message.visibility == .compact {
            compactBody
        } else {
            primaryBody
        }
    }

    @ViewBuilder
    private var primaryBody: some View {
        if usesPrimaryChrome {
            HStack(alignment: .top, spacing: 10) {
                Image(systemName: icon)
                    .font(.system(size: 14, weight: .semibold))
                    .frame(width: 22, height: 22)
                    .foregroundStyle(tint)
                    .background(iconBackground, in: RoundedRectangle(cornerRadius: 6))

                CodexMarkdownContent(message: message)
                    .foregroundStyle(foreground)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
            .padding(.horizontal, primaryHorizontalPadding)
            .padding(.vertical, primaryVerticalPadding)
            .background(primaryContainer, in: RoundedRectangle(cornerRadius: 8))
        } else {
            CodexMarkdownContent(message: message)
                .foregroundStyle(foreground)
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.horizontal, primaryHorizontalPadding)
                .padding(.vertical, primaryVerticalPadding)
        }
    }

    private var compactBody: some View {
        VStack(alignment: .leading, spacing: isRoutineCommandCompletion ? 4 : 6) {
            HStack(alignment: .top, spacing: 8) {
                Image(systemName: isRoutineCommandCompletion ? "checkmark.circle" : compactIcon)
                    .font(.system(size: isRoutineCommandCompletion ? 10 : 11, weight: .semibold))
                    .foregroundStyle(isRoutineCommandCompletion ? .tertiary : .secondary)
                    .frame(width: 16, height: 16)
                    .padding(.top, 2)

                if isCompactStatus {
                    Text(compactText.isEmpty ? (message.title ?? compactTitle) : compactText)
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                        .lineLimit(isExpanded ? nil : 2)
                        .frame(maxWidth: .infinity, alignment: .leading)
                } else {
                    VStack(alignment: .leading, spacing: 2) {
                        Text(message.title ?? compactTitle)
                            .font(isRoutineCommandCompletion ? .caption2 : .caption.weight(.semibold))
                            .foregroundStyle(isRoutineCommandCompletion ? .secondary : .primary)
                        if !compactText.isEmpty, !hidesCompactSecondaryText {
                            Text(compactText)
                                .font(.footnote)
                                .foregroundStyle(.secondary)
                                .lineLimit(isExpanded ? nil : 2)
                        }
                    }
                }

                Spacer(minLength: 8)

                if message.isStreaming {
                    ProgressView()
                        .controlSize(.mini)
                } else if hasCompactDetails {
                    Image(systemName: isExpanded ? "chevron.up" : "chevron.down")
                        .font(.caption2.weight(.semibold))
                        .foregroundStyle(.tertiary)
                        .frame(width: 18, height: 18)
                }
            }

            if isExpanded {
                if hidesCompactSecondaryText, !compactText.isEmpty {
                    Text(compactText)
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                } else if let detail = message.detail, !detail.isEmpty, detail != compactText {
                    Text(detail)
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                } else if isCompactStatus, rawCompactText != compactText {
                    Text(rawCompactText)
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                }
                if let transcript = message.transcript, !transcript.isEmpty {
                    Text(transcript)
                        .font(.system(.caption, design: .monospaced))
                        .textSelection(.enabled)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .padding(8)
                        .background(Color(.tertiarySystemBackground), in: RoundedRectangle(cornerRadius: 6))
                }
            }
        }
        .padding(.horizontal, 4)
        .padding(.vertical, isRoutineCommandCompletion ? 3 : 6)
        .contentShape(Rectangle())
        .onTapGesture {
            guard hasCompactDetails else { return }
            withAnimation(.easeInOut(duration: 0.16)) {
                isExpanded.toggle()
            }
        }
    }

    private var compactText: String {
        let rawText = rawCompactText
        guard isCompactStatus else {
            return rawText
        }
        return normalizedCompactStatusText(rawText)
    }

    private var rawCompactText: String {
        if !message.text.isEmpty {
            return message.text
        }
        return message.detail ?? ""
    }

    private func normalizedCompactStatusText(_ text: String) -> String {
        let trimmed = text.trimmingCharacters(in: .whitespacesAndNewlines)
        guard trimmed.hasPrefix("app-server sent non-JSON output") else {
            return text
        }
        if let openParen = trimmed.firstIndex(of: "("),
           let closeParen = trimmed[openParen...].firstIndex(of: ")") {
            return "Server output was not JSON \(trimmed[openParen...closeParen])"
        }
        return "Server output was not JSON"
    }

    private var isCompactStatus: Bool {
        message.kind == .status
    }

    private var hidesCompactSecondaryText: Bool {
        message.title?.trimmingCharacters(in: .whitespacesAndNewlines) == "Command completed"
    }

    private var isRoutineCommandCompletion: Bool {
        hidesCompactSecondaryText && !message.isStreaming
    }

    private var hasCompactDetails: Bool {
        if let transcript = message.transcript, !transcript.isEmpty {
            return true
        }
        if hidesCompactSecondaryText, !compactText.isEmpty {
            return true
        }
        if isCompactStatus, rawCompactText != compactText {
            return true
        }
        if let detail = message.detail, !detail.isEmpty, detail != compactText {
            return true
        }
        return false
    }

    private var compactTitle: String {
        switch message.kind {
        case .command:
            "Command"
        case .commandOutput:
            "Command output"
        case .fileChange:
            "File change"
        case .reasoningSummary:
            "Thinking"
        case .status:
            "Status"
        case .toolCall, .toolResult:
            "Tool"
        case .plan:
            "Plan"
        case .commentary, .finalAnswer:
            "Codex"
        case .userMessage:
            "You"
        }
    }

    private var compactIcon: String {
        switch message.kind {
        case .command, .commandOutput:
            "terminal"
        case .fileChange:
            "doc.text"
        case .reasoningSummary:
            "brain.head.profile"
        case .status:
            "info.circle"
        case .toolCall, .toolResult:
            "wrench.and.screwdriver"
        case .plan:
            "checklist"
        case .commentary, .finalAnswer:
            "sparkles"
        case .userMessage:
            "person"
        }
    }

    private var icon: String {
        switch message.role {
        case .user: "person.fill"
        case .assistant: "sparkles"
        case .status: "info.circle"
        case .tool: "wrench.and.screwdriver"
        case .commandOutput: "terminal"
        }
    }

    private var tint: Color {
        switch message.role {
        case .user: ShellowTheme.accent
        case .assistant: .green
        case .status: .secondary
        case .tool: .orange
        case .commandOutput: .purple
        }
    }

    private var primaryContainer: Color {
        switch message.role {
        case .user: ShellowTheme.accent.opacity(0.08)
        case .assistant, .status: .clear
        case .tool, .commandOutput: Color(.secondarySystemBackground)
        }
    }

    private var foreground: Color {
        message.role == .status ? .secondary : .primary
    }

    private var iconBackground: Color {
        message.role == .user ? tint.opacity(0.12) : .clear
    }

    private var usesPrimaryChrome: Bool {
        switch message.role {
        case .user, .tool, .commandOutput:
            true
        case .assistant, .status:
            false
        }
    }

    private var primaryHorizontalPadding: CGFloat {
        switch message.role {
        case .assistant, .status:
            4
        case .user, .tool, .commandOutput:
            10
        }
    }

    private var primaryVerticalPadding: CGFloat {
        switch message.role {
        case .assistant, .status:
            6
        case .user, .tool, .commandOutput:
            10
        }
    }

}

private extension CodexMessage {
    var isVisibleInChat: Bool {
        (visibility == .primary || visibility == .compact) && !isRoutineLifecycleStatus
    }

    private var isRoutineLifecycleStatus: Bool {
        guard kind == .status, visibility == .compact else {
            return false
        }
        let body = text.isEmpty ? (detail ?? "") : text
        return body.trimmingCharacters(in: .whitespacesAndNewlines) == "Codex thread resumed."
    }
}

private struct CodexMarkdownContent: View {
    let message: CodexMessage

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            if message.blocks.isEmpty {
                Text(message.text.isEmpty ? "..." : message.text)
                    .font(message.role == .commandOutput ? .system(.footnote, design: .monospaced) : .body)
                    .textSelection(.enabled)
            } else {
                ForEach(message.blocks) { block in
                    CodexMarkdownBlockView(block: block)
                }
            }

            if message.isStreaming {
                HStack(spacing: 6) {
                    ProgressView()
                        .controlSize(.mini)
                    Text("Streaming")
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                }
            }
        }
    }
}

private struct CodexMarkdownBlockView: View {
    let block: CodexMarkdownBlock

    var body: some View {
        switch block.kind {
        case .paragraph:
            Text(attributedRuns(block.runs, fallback: block.text, base: .body))
                .textSelection(.enabled)
        case .heading:
            Text(attributedRuns(block.runs, fallback: block.text, base: headingFont))
                .font(headingFont)
                .textSelection(.enabled)
                .padding(.top, block.level == 1 ? 4 : 2)
        case .list:
            VStack(alignment: .leading, spacing: 5) {
                ForEach(Array(block.items.enumerated()), id: \.offset) { index, item in
                    HStack(alignment: .top, spacing: 8) {
                        Text(block.ordered ? "\(index + 1)." : "•")
                            .font(.body.monospacedDigit())
                            .foregroundStyle(.secondary)
                            .frame(minWidth: 18, alignment: .trailing)
                        Text(attributedRuns(item.runs, fallback: item.text, base: .body))
                            .textSelection(.enabled)
                    }
                }
            }
        case .blockQuote:
            HStack(alignment: .top, spacing: 8) {
                Rectangle()
                    .fill(ShellowTheme.accent.opacity(0.45))
                    .frame(width: 3)
                    .clipShape(Capsule())
                Text(attributedRuns(block.runs, fallback: block.text, base: .body))
                    .foregroundStyle(.secondary)
                    .textSelection(.enabled)
            }
            .padding(.vertical, 2)
        case .codeBlock:
            CodexCodeBlockView(block: block)
        case .table:
            CodexTableBlockView(block: block)
        case .horizontalRule:
            Divider()
                .padding(.vertical, 4)
        case .image:
            CodexImageBlockView(block: block)
        }
    }

    private var headingFont: Font {
        switch block.level ?? 2 {
        case 1:
            .title3.weight(.semibold)
        case 2:
            .headline.weight(.semibold)
        default:
            .subheadline.weight(.semibold)
        }
    }
}

private struct CodexImageBlockView: View {
    let block: CodexMarkdownBlock

    private var urlText: String {
        block.imageUrl ?? block.text
    }

    private var altText: String {
        block.imageAlt ?? block.text
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            imageContent
                .frame(maxWidth: .infinity, alignment: .leading)

            if !altText.isEmpty {
                Text(altText)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(2)
            }
        }
    }

    @ViewBuilder
    private var imageContent: some View {
        if let image = localImage {
            imageView(image)
        } else if let url = URL(string: urlText),
                  url.scheme == "http" || url.scheme == "https" {
            AsyncImage(url: url) { phase in
                switch phase {
                case .empty:
                    imagePlaceholder("Loading image...")
                case .success(let image):
                    image
                        .resizable()
                        .scaledToFit()
                        .clipShape(RoundedRectangle(cornerRadius: 8))
                case .failure:
                    imagePlaceholder(urlText)
                @unknown default:
                    imagePlaceholder(urlText)
                }
            }
            .frame(maxHeight: 280)
        } else {
            imagePlaceholder(urlText)
        }
    }

    private var localImage: Image? {
        if let image = dataURLImage {
            return Image(uiImage: image)
        }

        let path: String
        if let url = URL(string: urlText), url.isFileURL {
            path = url.path
        } else if urlText.hasPrefix("/") || urlText.hasPrefix("~") {
            path = (urlText as NSString).expandingTildeInPath
        } else {
            return nil
        }

        guard let image = UIImage(contentsOfFile: path) else {
            return nil
        }
        return Image(uiImage: image)
    }

    private var dataURLImage: UIImage? {
        guard let comma = urlText.firstIndex(of: ","),
              urlText[..<comma].contains(";base64")
        else {
            return nil
        }
        let payload = String(urlText[urlText.index(after: comma)...])
        guard let data = Data(base64Encoded: payload) else {
            return nil
        }
        return UIImage(data: data)
    }

    private func imageView(_ image: Image) -> some View {
        image
            .resizable()
            .scaledToFit()
            .frame(maxHeight: 280)
            .clipShape(RoundedRectangle(cornerRadius: 8))
    }

    private func imagePlaceholder(_ text: String) -> some View {
        HStack(spacing: 8) {
            Image(systemName: "photo")
                .foregroundStyle(.secondary)
            Text(text.isEmpty ? "Image unavailable" : text)
                .font(.caption)
                .foregroundStyle(.secondary)
                .lineLimit(2)
        }
        .padding(10)
        .frame(maxWidth: .infinity, minHeight: 76, alignment: .leading)
        .background(Color(.tertiarySystemBackground), in: RoundedRectangle(cornerRadius: 8))
    }
}

private struct CodexTableBlockView: View {
    let block: CodexMarkdownBlock

    private var columnCount: Int {
        max(
            block.tableHeaders.count,
            block.tableRows.map(\.count).max() ?? 0,
            1
        )
    }

    var body: some View {
        ScrollView(.horizontal, showsIndicators: true) {
            VStack(alignment: .leading, spacing: 0) {
                if !block.tableHeaders.isEmpty {
                    CodexTableRowView(
                        cells: block.tableHeaders,
                        columnCount: columnCount,
                        isHeader: true
                    )
                }

                ForEach(Array(block.tableRows.enumerated()), id: \.offset) { _, row in
                    CodexTableRowView(
                        cells: row,
                        columnCount: columnCount,
                        isHeader: false
                    )
                }
            }
            .background(Color(.secondarySystemBackground), in: RoundedRectangle(cornerRadius: 8))
            .overlay(
                RoundedRectangle(cornerRadius: 8)
                    .stroke(Color(.separator).opacity(0.55), lineWidth: 0.5)
            )
        }
    }
}

private struct CodexTableRowView: View {
    let cells: [CodexMarkdownTableCell]
    let columnCount: Int
    let isHeader: Bool

    var body: some View {
        HStack(alignment: .top, spacing: 0) {
            ForEach(0..<columnCount, id: \.self) { index in
                let cell = index < cells.count ? cells[index] : CodexMarkdownTableCell(text: "", runs: [])
                Text(attributedRuns(cell.runs, fallback: cell.text, base: isHeader ? .caption.weight(.semibold) : .caption))
                    .lineLimit(nil)
                    .textSelection(.enabled)
                    .frame(width: 132, alignment: .topLeading)
                    .frame(minHeight: 34, alignment: .topLeading)
                    .padding(.horizontal, 9)
                    .padding(.vertical, 8)
                    .background(isHeader ? Color(.tertiarySystemBackground) : Color(.secondarySystemBackground))
                    .overlay(alignment: .trailing) {
                        Rectangle()
                            .fill(Color(.separator).opacity(0.45))
                            .frame(width: 0.5)
                    }
                    .overlay(alignment: .bottom) {
                        Rectangle()
                            .fill(Color(.separator).opacity(0.45))
                            .frame(height: 0.5)
                    }
            }
        }
    }
}

private struct CodexCodeBlockView: View {
    let block: CodexMarkdownBlock

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack(spacing: 8) {
                Text(block.language?.isEmpty == false ? block.language! : "code")
                    .font(.caption2.weight(.semibold))
                    .foregroundStyle(.secondary)
                if block.incomplete {
                    Text("streaming")
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                }
                Spacer()
                Button {
                    UIPasteboard.general.string = block.text
                } label: {
                    Image(systemName: "doc.on.doc")
                }
                .buttonStyle(.plain)
                .foregroundStyle(.secondary)
                .accessibilityLabel("Copy Code")
            }
            .padding(.horizontal, 10)
            .padding(.vertical, 7)
            .background(Color(.tertiarySystemBackground))

            ScrollView(.horizontal, showsIndicators: true) {
                Text(block.text.isEmpty ? " " : block.text)
                    .font(.system(.footnote, design: .monospaced))
                    .textSelection(.enabled)
                    .padding(10)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
        .background(Color(.secondarySystemBackground), in: RoundedRectangle(cornerRadius: 8))
        .clipShape(RoundedRectangle(cornerRadius: 8))
    }
}

private func attributedRuns(
    _ runs: [CodexMarkdownInlineRun],
    fallback: String,
    base: Font
) -> AttributedString {
    let usableRuns = runs.isEmpty
        ? [CodexMarkdownInlineRun(text: fallback, style: .text, url: nil)]
        : runs
    var output = AttributedString()
    for run in usableRuns {
        var piece = AttributedString(run.text)
        switch run.style {
        case .text:
            piece.font = base
        case .bold:
            piece.font = base.weight(.semibold)
        case .italic:
            piece.inlinePresentationIntent = .emphasized
        case .boldItalic:
            piece.font = base.weight(.semibold)
            piece.inlinePresentationIntent = .emphasized
        case .code:
            piece.font = .system(.body, design: .monospaced)
            piece.backgroundColor = Color(.tertiarySystemBackground)
        case .link:
            if let url = run.url.flatMap(URL.init(string:)) {
                piece.link = url
            }
            piece.foregroundColor = ShellowTheme.accent
            piece.underlineStyle = .single
        }
        output += piece
    }
    return output
}

private struct CodexApprovalRow: View {
    let approval: CodexApproval
    let decide: (String) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 9) {
            HStack(spacing: 8) {
                Image(systemName: "hand.raised")
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(.orange)
                Text(approval.title)
                    .font(.subheadline.weight(.semibold))
                Spacer()
            }

            Text(approval.detail)
                .font(.callout)
                .textSelection(.enabled)

            if let cwd = approval.cwd, !cwd.isEmpty {
                Text(cwd)
                    .font(.caption.monospaced())
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
                    .truncationMode(.middle)
            }

            HStack(spacing: 14) {
                Button {
                    decide("accept")
                } label: {
                    Label("Allow", systemImage: "checkmark")
                        .font(.caption.weight(.semibold))
                        .labelStyle(.titleAndIcon)
                        .padding(.vertical, 4)
                }
                .buttonStyle(.plain)
                .foregroundStyle(ShellowTheme.accent)

                Button {
                    decide("acceptForSession")
                } label: {
                    Text("Session")
                        .font(.caption.weight(.semibold))
                        .padding(.vertical, 4)
                }
                .buttonStyle(.plain)
                .foregroundStyle(.secondary)

                Spacer(minLength: 4)

                Button(role: .destructive) {
                    decide("decline")
                } label: {
                    Text("Deny")
                        .font(.caption.weight(.semibold))
                        .padding(.vertical, 4)
                }
                .buttonStyle(.plain)
            }
            .padding(.top, 2)
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 12)
        .background(Color(.secondarySystemBackground), in: RoundedRectangle(cornerRadius: 8))
        .overlay(alignment: .leading) {
            Rectangle()
                .fill(Color.orange.opacity(0.75))
                .frame(width: 3)
        }
        .clipShape(RoundedRectangle(cornerRadius: 8))
    }
}

private struct CodexSettingsSheet: View {
    @Binding var model: String
    let modelOptions: [CodexModelOption]
    let isLoadingModels: Bool
    let modelsError: String?
    @Binding var approvalPolicy: String
    @Binding var sandbox: String
    let canApply: Bool
    let apply: () -> Void

    @Environment(\.dismiss) private var dismiss

    private var pickerOptions: [CodexModelOption] {
        var options = modelOptions
        if let model = normalizeModel(model),
           !options.contains(where: { $0.id == model }) {
            options.append(CodexModelOption(id: model, name: model))
        }
        return options
    }

    var body: some View {
        NavigationStack {
            Form {
                Section("Model") {
                    Picker("Model", selection: $model) {
                        Text("Use Codex default").tag("")
                        ForEach(pickerOptions) { option in
                            Text(option.name).tag(option.id)
                        }
                    }
                    if isLoadingModels {
                        CodexInlineStatusRow(text: "Loading models", isLoading: true)
                    } else if let modelsError {
                        CodexInlineStatusRow(text: modelsError, tone: .warning)
                    }
                }

                Section("Approval") {
                    Picker("Policy", selection: $approvalPolicy) {
                        Text("Default").tag("")
                        Text("Untrusted").tag("untrusted")
                        Text("On failure").tag("on-failure")
                        Text("On request").tag("on-request")
                        Text("Never").tag("never")
                    }
                }

                Section("Sandbox") {
                    Picker("Mode", selection: $sandbox) {
                        Text("Default").tag("")
                        Text("Read only").tag("read-only")
                        Text("Workspace write").tag("workspace-write")
                        Text("Danger full access").tag("danger-full-access")
                    }
                }
            }
            .navigationTitle("Codex Settings")
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") {
                        dismiss()
                    }
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button("Apply") {
                        apply()
                        dismiss()
                    }
                    .disabled(!canApply)
                }
            }
        }
    }
}

private extension CodexThreadSummary {
    var displayTitle: String {
        if let name = name?.trimmingCharacters(in: .whitespacesAndNewlines), !name.isEmpty {
            return name
        }
        let preview = preview.trimmingCharacters(in: .whitespacesAndNewlines)
        return preview.isEmpty ? id : preview
    }
}

private func mergeProjects(_ groups: [String]...) -> [String] {
    var result: [String] = []
    for group in groups {
        for path in group {
            let path = path.trimmingCharacters(in: .whitespacesAndNewlines)
            guard !path.isEmpty, !result.contains(path) else { continue }
            result.append(path)
        }
    }
    return Array(result.prefix(20))
}

private func normalizeModel(_ value: String?) -> String? {
    guard let value = value?.trimmingCharacters(in: .whitespacesAndNewlines),
          !value.isEmpty
    else {
        return nil
    }
    return value
}

private func lastPathComponent(_ path: String) -> String {
    let trimmed = path.trimmingCharacters(in: CharacterSet(charactersIn: "/"))
    return trimmed.split(separator: "/").last.map(String.init) ?? path
}

private func codexCompactPath(_ path: String) -> String {
    let trimmed = path.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !trimmed.isEmpty else {
        return path
    }

    let components = trimmed
        .trimmingCharacters(in: CharacterSet(charactersIn: "/"))
        .split(separator: "/")
        .map(String.init)
        .filter { !$0.isEmpty }
    guard !components.isEmpty else {
        return trimmed
    }

    if components.count >= 2, components[0] == "Users" {
        let remainder = components.dropFirst(2)
        return remainder.isEmpty ? "~" : "~/" + remainder.joined(separator: "/")
    }

    return trimmed.hasPrefix("/") ? "/" + components.joined(separator: "/") : components.joined(separator: "/")
}

func privateKeyLooksUsable(_ value: String) -> Bool {
    value.contains("BEGIN") && value.contains("PRIVATE KEY")
}

private struct HostProfileRow: View {
    let profile: HostProfile
    let open: () -> Void

    var body: some View {
        Button(action: open) {
            HStack(spacing: 12) {
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

                Image(systemName: "chevron.right")
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(.tertiary)
            }
            .padding(.vertical, 6)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
    }
}

#Preview {
    HostsScreen(
        profiles: .constant(HostProfile.samples),
        sshKeys: .constant([]),
        onOpenSettings: {},
        connectTerminal: { _ in },
        connectCodex: { _ in }
    )
}
