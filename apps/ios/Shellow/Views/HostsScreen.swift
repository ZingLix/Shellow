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

    var body: some View {
        List {
            Section("Hosts") {
                ForEach(profiles) { profile in
                    HostProfileRow(
                        profile: profile,
                        connectTerminal: {
                            connectTerminal(profile)
                        },
                        connectCodex: {
                            connectCodex(profile)
                        }
                    )
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
                HStack {
                    Button {
                        isManagingKeys = true
                    } label: {
                        Image(systemName: "key")
                    }
                    .accessibilityLabel("Manage Keys")

                    Button {
                        isAddingProfile = true
                    } label: {
                        Image(systemName: "plus")
                    }
                    .accessibilityLabel("Add Host")
                }
            }
        }
        .sheet(isPresented: $isAddingProfile) {
            NewHostProfileSheet(
                draftName: $draftName,
                draftHost: $draftHost,
                draftPort: $draftPort,
                draftUser: $draftUser,
                addProfile: addProfile
            )
            .presentationDetents([.medium])
        }
        .sheet(isPresented: $isManagingKeys) {
            SSHKeyManagementSheet(
                credentials: $sshKeys
            )
            .presentationDetents([.large])
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
                Section("Host") {
                    LabeledContent("Endpoint", value: profile.endpoint)
                    LabeledContent("Host Key", value: profile.hostKeyTrustTitle)
                    if let reason {
                        Text(reason)
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                    }
                }

                Section("Password") {
                    SecureField("Password", text: $password)
                        .textContentType(.password)
                    Toggle("Save password in Keychain", isOn: $rememberPassword)
                    if let keychainStatus {
                        Text(keychainStatus)
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
                        Text("No private keys saved")
                            .foregroundStyle(.secondary)
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
                    TextEditor(text: $privateKey)
                        .font(.system(.footnote, design: .monospaced))
                        .frame(minHeight: 180)
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()
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
        !name.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty &&
            privateKeyLooksUsable(privateKey)
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
            Label(message, systemImage: "exclamationmark.triangle")
                .font(.caption)
                .foregroundStyle(.orange)
                .lineLimit(2)
                .padding(.horizontal, 14)
                .padding(.vertical, 8)
                .frame(maxWidth: .infinity, alignment: .leading)
                .background(Color.orange.opacity(0.10))
        } else if let message = snapshot.operation.lastSuccess {
            Label(message, systemImage: "checkmark.circle")
                .font(.caption)
                .foregroundStyle(.green)
                .lineLimit(1)
                .padding(.horizontal, 14)
                .padding(.vertical, 8)
                .frame(maxWidth: .infinity, alignment: .leading)
                .background(Color.green.opacity(0.08))
        }
    }

    private var codexHeader: some View {
        HStack(spacing: 10) {
            Button {
                goBack()
            } label: {
                Image(systemName: "chevron.left")
            }
            .accessibilityLabel("Back")

            VStack(alignment: .leading, spacing: 2) {
                Text(snapshot.title)
                    .font(.headline)
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

            if let onReconnect {
                Button(action: onReconnect) {
                    Image(systemName: "arrow.clockwise")
                }
                .accessibilityLabel("Reconnect Codex")
            }

            Button(action: onDisconnect) {
                Image(systemName: "power")
            }
            .accessibilityLabel("Disconnect Codex")
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 10)
    }

    private var headerSubtitle: String {
        let cwd = snapshot.cwd.map { "  \($0)" } ?? ""
        return "\(snapshot.status.title)  \(snapshot.endpoint)\(cwd)"
    }

    private var chatView: some View {
        ScrollViewReader { proxy in
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 10) {
                    chatToolbar

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

                    if snapshot.turnActive {
                        HStack(spacing: 8) {
                            ProgressView()
                                .controlSize(.small)
                            Text("Codex is working")
                                .font(.caption)
                                .foregroundStyle(.secondary)
                            Spacer()
                            Button(role: .destructive, action: onInterruptTurn) {
                                Label("Interrupt", systemImage: "stop.fill")
                            }
                            .buttonStyle(.bordered)
                            .labelStyle(.iconOnly)
                            .accessibilityLabel("Interrupt Codex Turn")
                        }
                        .padding(.vertical, 6)
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
            .onChange(of: snapshot.messages.count) {
                scrollToChatBottom(proxy, animated: true)
            }
            .onChange(of: snapshot.pendingApprovals.count) {
                scrollToChatBottom(proxy, animated: true)
            }
            .onChange(of: snapshot.turnActive) {
                scrollToChatBottom(proxy, animated: true)
            }
        }
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

    private var chatToolbar: some View {
        HStack(spacing: 8) {
            VStack(alignment: .leading, spacing: 2) {
                Text(snapshot.threadDetail.thread?.displayTitle ?? "Active Thread")
                    .font(.subheadline.weight(.semibold))
                    .lineLimit(1)
                Text(snapshot.cwd ?? snapshot.threadDetail.thread?.cwd ?? "No project")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }

            Spacer()

            if let threadId = snapshot.threadId,
               let cursor = snapshot.threadDetail.turnsNextCursor,
               !cursor.isEmpty {
                Button {
                    Task { await onLoadMoreThreadTurns(threadId, cursor) }
                } label: {
                    Image(systemName: "clock.arrow.circlepath")
                }
                .buttonStyle(.bordered)
                .disabled(snapshot.threadDetail.isLoadingMore)
                .accessibilityLabel("Load More Thread History")
            }

            if let threadId = snapshot.threadId {
                Button {
                    Task { await onForkThread(threadId, selectedProjectPath) }
                } label: {
                    Image(systemName: "arrow.triangle.branch")
                }
                .buttonStyle(.bordered)
                .accessibilityLabel("Fork Thread")
            }
        }
        .padding(10)
        .background(Color(.secondarySystemBackground), in: RoundedRectangle(cornerRadius: 8))
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
                    projectsSection
                    recentConversationsSection
                }
                .padding(14)
            }

            Divider()
            codexHomeSearchBar
        }
    }

    private var projectThreadsView: some View {
        VStack(spacing: 0) {
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 14) {
                    projectPageHeader
                    projectConversationsSection
                }
                .padding(14)
            }

            Divider()
            projectSearchBar
        }
    }

    private var draftChatView: some View {
        VStack(spacing: 0) {
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 14) {
                    draftChatHeader
                    draftWorkspaceSection
                }
                .padding(14)
            }

            Divider()
            draftComposer
        }
    }

    private var projectPageHeader: some View {
        VStack(alignment: .leading, spacing: 10) {
            Button {
                homeRoute = .overview
                historyScope = .allProjects
                Task { await refreshHistory() }
            } label: {
                Label("Projects", systemImage: "chevron.left")
            }
            .buttonStyle(.bordered)

            CodexSectionHeader(
                title: lastPathComponent(selectedProjectPath),
                detail: selectedProjectPath
            )
        }
    }

    private var draftChatHeader: some View {
        VStack(alignment: .leading, spacing: 10) {
            Button {
                homeRoute = draftReturnRoute
            } label: {
                Label(draftReturnRoute == .project ? lastPathComponent(selectedProjectPath) : "Projects", systemImage: "chevron.left")
            }
            .buttonStyle(.bordered)

            CodexSectionHeader(
                title: "New Conversation",
                detail: selectedProjectPath.isEmpty ? "Choose a workspace before sending" : selectedProjectPath
            )
        }
    }

    private var draftWorkspaceSection: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack(spacing: 8) {
                TextField("Workspace path", text: $selectedPath)
                    .textFieldStyle(.roundedBorder)
                    .textInputAutocapitalization(.never)
                    .autocorrectionDisabled()

                Button {
                    homeRoute = .project
                    Task { await selectTypedProject() }
                } label: {
                    Image(systemName: "folder")
                }
                .buttonStyle(.bordered)
                .disabled(!canUseProjectActions)
                .accessibilityLabel("Show Workspace Conversations")
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
            HStack(spacing: 8) {
                CodexSectionHeader(title: "Conversations", detail: "Current project")

                Spacer()

                Button {
                    showArchivedThreads.toggle()
                    Task { await refreshHistory() }
                } label: {
                    Image(systemName: showArchivedThreads ? "archivebox.fill" : "archivebox")
                }
                .buttonStyle(.bordered)
                .accessibilityLabel("Toggle Archived Threads")

                Button {
                    Task { await refreshHistory() }
                } label: {
                    Image(systemName: "arrow.clockwise")
                }
                .buttonStyle(.bordered)
                .disabled(!canUseProjectActions)
                .accessibilityLabel("Refresh Conversations")
            }

            if snapshot.threads.isLoading {
                ProgressView()
                    .controlSize(.small)
                    .padding(.vertical, 8)
            }

            if let error = snapshot.threads.error {
                Label(error, systemImage: "exclamationmark.triangle")
                    .font(.callout)
                    .foregroundStyle(.orange)
                    .padding(10)
                    .background(Color.orange.opacity(0.12), in: RoundedRectangle(cornerRadius: 8))
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
                Button {
                    Task { await loadMoreHistory(cursor: nextCursor) }
                } label: {
                    if snapshot.threads.isLoadingMore {
                        ProgressView()
                            .controlSize(.small)
                    } else {
                        Label("Load More", systemImage: "chevron.down")
                    }
                }
                .buttonStyle(.bordered)
                .frame(maxWidth: .infinity)
                .disabled(snapshot.threads.isLoadingMore)
            }

            if visibleThreads.isEmpty,
               !snapshot.threads.isLoading,
               snapshot.threads.error == nil {
                Text(homeSearchTerm.isEmpty ? "No conversations in this project" : "No matching conversations")
                    .font(.callout)
                    .foregroundStyle(.secondary)
                    .padding(.vertical, 20)
                    .frame(maxWidth: .infinity)
            }
        }
    }

    private var projectsSection: some View {
        VStack(alignment: .leading, spacing: 10) {
            CodexSectionHeader(title: "Projects", detail: selectedProjectPath.isEmpty ? "No project selected" : selectedProjectPath)

            HStack(spacing: 8) {
                TextField("Project path", text: $selectedPath)
                    .textFieldStyle(.roundedBorder)
                    .textInputAutocapitalization(.never)
                    .autocorrectionDisabled()

                Button {
                    Task { await selectTypedProject() }
                } label: {
                    Image(systemName: "line.3.horizontal.decrease.circle")
                }
                .buttonStyle(.bordered)
                .disabled(!canUseProjectActions)
                .accessibilityLabel("Show Project Conversations")
            }

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
                Text(homeSearchTerm.isEmpty ? "No projects" : "No matching projects")
                    .font(.callout)
                    .foregroundStyle(.secondary)
                    .padding(.vertical, 20)
                    .frame(maxWidth: .infinity)
            }
        }
    }

    private var recentConversationsSection: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack(spacing: 8) {
                CodexSectionHeader(title: "Recent Conversations", detail: historyScopeTitle)

                Spacer()

                Menu {
                    Picker("Scope", selection: $historyScope) {
                        Text("Current Project").tag(CodexHistoryScope.currentProject)
                        Text("All Projects").tag(CodexHistoryScope.allProjects)
                    }
                } label: {
                    Image(systemName: "line.3.horizontal.decrease.circle")
                }
                .accessibilityLabel("Conversation Scope")

                Button {
                    showArchivedThreads.toggle()
                    Task { await refreshHistory() }
                } label: {
                    Image(systemName: showArchivedThreads ? "archivebox.fill" : "archivebox")
                }
                .buttonStyle(.bordered)
                .accessibilityLabel("Toggle Archived Threads")

                Button {
                    Task { await refreshHistory() }
                } label: {
                    Image(systemName: "arrow.clockwise")
                }
                .buttonStyle(.bordered)
                .disabled(!canUseHistoryActions)
                .accessibilityLabel("Refresh Conversations")
            }

            if snapshot.threads.isLoading {
                ProgressView()
                    .controlSize(.small)
                    .padding(.vertical, 8)
            }

            if let error = snapshot.threads.error {
                Label(error, systemImage: "exclamationmark.triangle")
                    .font(.callout)
                    .foregroundStyle(.orange)
                    .padding(10)
                    .background(Color.orange.opacity(0.12), in: RoundedRectangle(cornerRadius: 8))
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
                Button {
                    Task { await loadMoreHistory(cursor: nextCursor) }
                } label: {
                    if snapshot.threads.isLoadingMore {
                        ProgressView()
                            .controlSize(.small)
                    } else {
                        Label("Load More", systemImage: "chevron.down")
                    }
                }
                .buttonStyle(.bordered)
                .frame(maxWidth: .infinity)
                .disabled(snapshot.threads.isLoadingMore)
            }

            if visibleThreads.isEmpty,
               !snapshot.threads.isLoading,
               snapshot.threads.error == nil {
                Text(homeSearchTerm.isEmpty ? "No recent conversations" : "No matching conversations")
                    .font(.callout)
                    .foregroundStyle(.secondary)
                    .padding(.vertical, 20)
                    .frame(maxWidth: .infinity)
            }
        }
    }

    private var codexHomeSearchBar: some View {
        HStack(alignment: .bottom, spacing: 10) {
            modelSettingsButton

            HStack(spacing: 8) {
                Image(systemName: "magnifyingglass")
                    .foregroundStyle(.secondary)
                TextField("Search projects or conversations", text: $historySearch)
                    .textInputAutocapitalization(.never)
                    .autocorrectionDisabled()
                    .submitLabel(.search)
                    .onSubmit {
                        Task { await refreshHistory() }
                    }
            }
            .padding(.horizontal, 10)
            .padding(.vertical, 9)
            .background(Color(.secondarySystemBackground), in: RoundedRectangle(cornerRadius: 8))
            .frame(maxWidth: .infinity)

            Button {
                beginDraftChat()
            } label: {
                Label("Chat", systemImage: "bubble.left.and.text.bubble.right.fill")
            }
            .buttonStyle(.borderedProminent)
            .disabled(!canUseProjectActions)
            .accessibilityLabel("Enter Codex Chat")
        }
        .padding(12)
        .background(.bar)
    }

    private var projectSearchBar: some View {
        HStack(alignment: .bottom, spacing: 10) {
            modelSettingsButton

            HStack(spacing: 8) {
                Image(systemName: "magnifyingglass")
                    .foregroundStyle(.secondary)
                TextField("Search this project", text: $historySearch)
                    .textInputAutocapitalization(.never)
                    .autocorrectionDisabled()
                    .submitLabel(.search)
                    .onSubmit {
                        Task { await refreshHistory() }
                    }
            }
            .padding(.horizontal, 10)
            .padding(.vertical, 9)
            .background(Color(.secondarySystemBackground), in: RoundedRectangle(cornerRadius: 8))
            .frame(maxWidth: .infinity)

            Button {
                beginDraftChat()
            } label: {
                Label("Chat", systemImage: "bubble.left.and.text.bubble.right.fill")
            }
            .buttonStyle(.borderedProminent)
            .disabled(!canUseProjectActions)
            .accessibilityLabel("Start Chat In Project")
        }
        .padding(12)
        .background(.bar)
    }

    private var composer: some View {
        HStack(alignment: .bottom, spacing: 10) {
            modelSettingsButton

            TextField("Message Codex", text: $draft, axis: .vertical)
                .lineLimit(1...5)
                .textFieldStyle(.roundedBorder)
                .textInputAutocapitalization(.sentences)

            Button {
                sendDraft()
            } label: {
                Image(systemName: snapshot.turnActive ? "arrow.turn.down.right" : "paperplane.fill")
            }
            .buttonStyle(.borderedProminent)
            .disabled(!canSend)
            .accessibilityLabel("Send")
        }
        .padding(12)
        .background(.bar)
    }

    private var draftComposer: some View {
        HStack(alignment: .bottom, spacing: 10) {
            modelSettingsButton

            TextField("Message Codex", text: $draft, axis: .vertical)
                .lineLimit(1...5)
                .textFieldStyle(.roundedBorder)
                .textInputAutocapitalization(.sentences)

            Button {
                Task { await sendInitialDraft() }
            } label: {
                if isStartingDraftThread {
                    ProgressView()
                        .controlSize(.small)
                } else {
                    Image(systemName: "paperplane.fill")
                }
            }
            .buttonStyle(.borderedProminent)
            .disabled(!canSendInitialDraft)
            .accessibilityLabel("Send")
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

    private var modelSettingsButton: some View {
        Button {
            settingsModel = snapshot.settings.model ?? ""
            settingsApprovalPolicy = snapshot.settings.approvalPolicy ?? ""
            settingsSandbox = snapshot.settings.sandbox ?? ""
            showingSettings = true
        } label: {
            HStack(spacing: 6) {
                Image(systemName: "slider.horizontal.3")
                Text(selectedModelTitle)
                    .lineLimit(1)
                    .truncationMode(.tail)
            }
        }
        .buttonStyle(.bordered)
        .frame(maxWidth: 145)
        .accessibilityLabel("Codex Settings")
    }

    private var selectedProjectPath: String {
        selectedPath.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    private var homeSearchTerm: String {
        historySearch.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    private var historyScopeTitle: String {
        switch historyScope {
        case .currentProject:
            "Current project"
        case .allProjects:
            "All projects"
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
        let path = selectedProjectPath
        guard !path.isEmpty else { return }
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
    let detail: String

    var body: some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(title)
                .font(.headline)
                .foregroundStyle(.primary)
            Text(detail)
                .font(.caption)
                .foregroundStyle(.secondary)
                .lineLimit(1)
        }
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
                    .font(.system(size: 15, weight: .semibold))
                    .frame(width: 28, height: 28)
                    .foregroundStyle(ShellowTheme.accent)
                    .background(ShellowTheme.accent.opacity(0.12), in: RoundedRectangle(cornerRadius: 6))

                VStack(alignment: .leading, spacing: 2) {
                    Text(title)
                        .font(.body.weight(.medium))
                        .foregroundStyle(.primary)
                    Text(subtitle)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }

                Spacer()

                Image(systemName: "chevron.right")
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(.tertiary)
            }
            .padding(10)
            .background(Color(.secondarySystemBackground), in: RoundedRectangle(cornerRadius: 8))
        }
        .buttonStyle(.plain)
    }
}

private struct CodexThreadRow: View {
    let thread: CodexThreadSummary
    let archived: Bool
    let isOpening: Bool
    let resume: () -> Void
    let rename: () -> Void
    let fork: () -> Void
    let archive: () -> Void
    let unarchive: () -> Void
    let delete: () -> Void

    var body: some View {
        HStack(alignment: .top, spacing: 8) {
            Button(action: resume) {
                HStack(alignment: .top, spacing: 10) {
                    Image(systemName: "bubble.left.and.text.bubble.right")
                        .font(.system(size: 15, weight: .semibold))
                        .frame(width: 28, height: 28)
                        .foregroundStyle(.green)
                        .background(Color.green.opacity(0.12), in: RoundedRectangle(cornerRadius: 6))

                    VStack(alignment: .leading, spacing: 4) {
                        Text(thread.displayTitle)
                            .font(.body.weight(.semibold))
                            .foregroundStyle(.primary)
                            .lineLimit(2)
                        Text(thread.cwd)
                            .font(.caption)
                            .foregroundStyle(.secondary)
                            .lineLimit(1)
                        Text(historyMeta)
                            .font(.caption2)
                            .foregroundStyle(.tertiary)
                    }

                    Spacer(minLength: 0)
                }
            }
            .buttonStyle(.plain)
            .frame(maxWidth: .infinity, alignment: .leading)
            .disabled(isOpening)

            if isOpening {
                ProgressView()
                    .controlSize(.small)
                    .frame(width: 32, height: 32)
            } else {
                Button(action: resume) {
                    Image(systemName: "arrow.forward.circle")
                        .font(.system(size: 19, weight: .semibold))
                        .frame(width: 32, height: 32)
                }
                .buttonStyle(.plain)
                .foregroundStyle(ShellowTheme.accent)
                .accessibilityLabel("Open Thread")
            }

            Menu {
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
            } label: {
                Image(systemName: "ellipsis.circle")
                    .font(.system(size: 19, weight: .semibold))
                    .frame(width: 32, height: 32)
                    .foregroundStyle(.secondary)
            }
            .buttonStyle(.plain)
            .accessibilityLabel("Thread Actions")
        }
        .padding(10)
        .background(Color(.secondarySystemBackground), in: RoundedRectangle(cornerRadius: 8))
    }

    private var historyMeta: String {
        let date = Date(timeIntervalSince1970: TimeInterval(thread.updatedAt))
        var parts = [
            date.formatted(date: .abbreviated, time: .shortened),
            thread.status
        ]
        if thread.forkedFromId != nil {
            parts.append("fork")
        }
        return parts.joined(separator: "  ")
    }
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

    private var primaryBody: some View {
        HStack(alignment: .top, spacing: 10) {
            Image(systemName: icon)
                .font(.system(size: 14, weight: .semibold))
                .frame(width: 24, height: 24)
                .foregroundStyle(tint)
                .background(tint.opacity(0.12), in: RoundedRectangle(cornerRadius: 6))

            CodexMarkdownContent(message: message)
                .foregroundStyle(foreground)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(10)
        .background(background, in: RoundedRectangle(cornerRadius: 8))
    }

    private var compactBody: some View {
        VStack(alignment: .leading, spacing: 7) {
            HStack(alignment: .top, spacing: 8) {
                Text(compactGlyph)
                    .font(.caption.weight(.semibold).monospaced())
                    .foregroundStyle(.secondary)
                    .frame(width: 18)

                VStack(alignment: .leading, spacing: 2) {
                    Text(message.title ?? compactTitle)
                        .font(.caption.weight(.semibold))
                        .foregroundStyle(.primary)
                    if !compactText.isEmpty {
                        Text(compactText)
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .lineLimit(isExpanded ? nil : 2)
                    }
                }

                Spacer(minLength: 8)

                if message.isStreaming {
                    ProgressView()
                        .controlSize(.mini)
                } else if hasCompactDetails {
                    Text(isExpanded ? "Hide" : "Details")
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                }
            }

            if isExpanded {
                if let detail = message.detail, !detail.isEmpty, detail != compactText {
                    Text(detail)
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
        .padding(10)
        .background(Color(.secondarySystemBackground).opacity(0.72), in: RoundedRectangle(cornerRadius: 8))
        .contentShape(Rectangle())
        .onTapGesture {
            guard hasCompactDetails else { return }
            withAnimation(.easeInOut(duration: 0.16)) {
                isExpanded.toggle()
            }
        }
    }

    private var compactText: String {
        if !message.text.isEmpty {
            return message.text
        }
        return message.detail ?? ""
    }

    private var hasCompactDetails: Bool {
        if let transcript = message.transcript, !transcript.isEmpty {
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

    private var compactGlyph: String {
        switch message.kind {
        case .command, .commandOutput:
            "$"
        case .fileChange:
            "+"
        case .reasoningSummary:
            "..."
        case .status:
            "i"
        case .toolCall, .toolResult:
            ">"
        case .plan:
            "#"
        case .commentary, .finalAnswer:
            "*"
        case .userMessage:
            "@"
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

    private var background: Color {
        switch message.role {
        case .user: ShellowTheme.accent.opacity(0.08)
        case .assistant: Color(.secondarySystemBackground)
        case .status: Color(.tertiarySystemBackground)
        case .tool, .commandOutput: Color(.secondarySystemBackground)
        }
    }

    private var foreground: Color {
        message.role == .status ? .secondary : .primary
    }

}

private extension CodexMessage {
    var isVisibleInChat: Bool {
        visibility == .primary || visibility == .compact
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
        VStack(alignment: .leading, spacing: 10) {
            HStack(spacing: 8) {
                Image(systemName: "hand.raised.fill")
                    .foregroundStyle(.orange)
                Text(approval.title)
                    .font(.headline)
                Spacer()
            }

            Text(approval.detail)
                .font(.callout)
                .textSelection(.enabled)

            if let cwd = approval.cwd, !cwd.isEmpty {
                Text(cwd)
                    .font(.caption.monospaced())
                    .foregroundStyle(.secondary)
            }

            HStack {
                Button {
                    decide("accept")
                } label: {
                    Label("Allow", systemImage: "checkmark")
                }
                .buttonStyle(.borderedProminent)

                Button {
                    decide("acceptForSession")
                } label: {
                    Label("Session", systemImage: "checkmark.seal")
                }
                .buttonStyle(.bordered)

                Button(role: .destructive) {
                    decide("decline")
                } label: {
                    Label("Deny", systemImage: "xmark")
                }
                .buttonStyle(.bordered)
            }
            .labelStyle(.iconOnly)
        }
        .padding(12)
        .background(Color.orange.opacity(0.12), in: RoundedRectangle(cornerRadius: 8))
    }
}

private struct CodexSettingsSheet: View {
    @Binding var model: String
    let modelOptions: [CodexModelOption]
    let isLoadingModels: Bool
    let modelsError: String?
    @Binding var approvalPolicy: String
    @Binding var sandbox: String
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
                        ProgressView()
                            .controlSize(.small)
                    } else if let modelsError {
                        Text(modelsError)
                            .font(.caption)
                            .foregroundStyle(.secondary)
                            .lineLimit(2)
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

func privateKeyLooksUsable(_ value: String) -> Bool {
    value.contains("BEGIN") && value.contains("PRIVATE KEY")
}

private struct HostProfileRow: View {
    let profile: HostProfile
    let connectTerminal: () -> Void
    let connectCodex: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
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
            }

            HStack(spacing: 10) {
                Button(action: connectTerminal) {
                    Label("Terminal", systemImage: "terminal")
                        .frame(maxWidth: .infinity)
                }
                .buttonStyle(.bordered)

                Button(action: connectCodex) {
                    Label("Codex", systemImage: "bubble.left.and.text.bubble.right")
                        .frame(maxWidth: .infinity)
                }
                .buttonStyle(.borderedProminent)
            }
        }
        .padding(.vertical, 6)
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
