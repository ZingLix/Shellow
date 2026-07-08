import Foundation
import SwiftUI
import UIKit

private struct TerminalSelectionPoint: Equatable {
    static let lineEndColumn = Int.max / 4

    var row: Int
    var column: Int
}

private enum TerminalSelection: Equatable {
    case grid(anchor: TerminalSelectionPoint, focus: TerminalSelectionPoint)
    case history(anchor: Int, focus: Int)
}

private struct PendingPaste: Identifiable {
    let id = UUID()
    let text: String

    var summary: String {
        let lines = max(1, text.lineCount)
        return "Send \(lines) lines and \(text.count) characters to the terminal?"
    }
}

private struct PendingRemoteClipboard: Identifiable {
    let id: Int
    let text: String

    var summary: String {
        let lines = max(1, text.lineCount)
        return "Copy \(lines) lines and \(text.count) characters from the remote terminal?"
    }
}

private struct TranscriptSaveResult: Identifiable {
    let id = UUID()
    let title: String
    let message: String
}

private enum TerminalSearchHit: Hashable {
    case grid(row: Int, start: Int, end: Int)
    case history(Int)

    var id: String {
        switch self {
        case .grid(let row, let start, let end): "grid-search-\(row)-\(start)-\(end)"
        case .history(let row): "history-search-\(row)"
        }
    }

    var gridRow: Int? {
        guard case .grid(let row, _, _) = self else { return nil }
        return row
    }

    var gridRange: Range<Int>? {
        guard case .grid(_, let start, let end) = self else { return nil }
        return start..<end
    }

    static func gridRowID(_ row: Int) -> String {
        "grid-row-\(row)"
    }

    static func historyRowID(_ row: Int) -> String {
        "history-row-\(row)"
    }
}

private struct TerminalSearchPresentation: Equatable {
    var query: String
    var hits: [TerminalSearchHit]
    var activeHit: TerminalSearchHit?

    var isEmpty: Bool {
        query.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
    }

    var activeOrdinal: Int {
        guard let activeHit, let index = hits.firstIndex(of: activeHit) else { return 0 }
        return index + 1
    }

    var activeGridRow: Int? {
        activeHit?.gridRow
    }

    var activeGridRange: Range<Int>? {
        activeHit?.gridRange
    }

    func containsGrid(row: Int) -> Bool {
        hits.contains { $0.gridRow == row }
    }

    func containsHistory(row: Int) -> Bool {
        hits.contains(.history(row))
    }

    func gridRanges(row: Int) -> [Range<Int>] {
        hits.compactMap { hit in
            guard hit.gridRow == row else { return nil }
            return hit.gridRange
        }
    }
}

private struct TerminalKeyboardAvoidanceState: Equatable {
    static let hidden = TerminalKeyboardAvoidanceState(
        endFrame: .null,
        animationDuration: 0.25,
        animationCurveRaw: UIView.AnimationCurve.easeInOut.rawValue
    )

    let endFrame: CGRect
    let animationDuration: TimeInterval
    let animationCurveRaw: Int

    init(
        endFrame: CGRect,
        animationDuration: TimeInterval,
        animationCurveRaw: Int
    ) {
        self.endFrame = endFrame
        self.animationDuration = animationDuration
        self.animationCurveRaw = animationCurveRaw
    }

    init(notification: Notification, forceHidden: Bool = false) {
        let userInfo = notification.userInfo ?? [:]
        let frame = (userInfo[UIResponder.keyboardFrameEndUserInfoKey] as? NSValue)?.cgRectValue ?? .null
        let duration = (userInfo[UIResponder.keyboardAnimationDurationUserInfoKey] as? NSNumber)?.doubleValue ?? 0.25
        let curve = (userInfo[UIResponder.keyboardAnimationCurveUserInfoKey] as? NSNumber)?.intValue
            ?? UIView.AnimationCurve.easeInOut.rawValue

        self.endFrame = forceHidden ? .null : frame
        self.animationDuration = duration
        self.animationCurveRaw = curve
    }

    var animation: Animation {
        let duration = max(0.01, animationDuration)
        switch UIView.AnimationCurve(rawValue: animationCurveRaw) {
        case .easeIn:
            return .easeIn(duration: duration)
        case .easeOut:
            return .easeOut(duration: duration)
        case .linear:
            return .linear(duration: duration)
        default:
            return .easeInOut(duration: duration)
        }
    }

    func bottomInset(overlapping viewFrame: CGRect) -> CGFloat {
        guard !endFrame.isNull, !endFrame.isEmpty, !viewFrame.isNull, !viewFrame.isEmpty else {
            return 0
        }
        guard endFrame.minY < viewFrame.maxY else { return 0 }
        return max(0, viewFrame.maxY - endFrame.minY)
    }

    func cursorAwareOffset(
        overlapping viewFrame: CGRect,
        cursorBottomY: CGFloat?,
        coveredBottomInset: CGFloat,
        bottomPadding: CGFloat
    ) -> CGFloat {
        guard let cursorBottomY, coveredBottomInset > 0 else { return 0 }
        guard !viewFrame.isNull, !viewFrame.isEmpty else { return 0 }

        let visibleBottomY = max(0, viewFrame.height - coveredBottomInset)
        return max(0, cursorBottomY + bottomPadding - visibleBottomY)
    }
}

struct TerminalScreen: View {
    @Environment(\.dismiss) private var dismiss
    @Binding var session: TerminalSession
    let settings: ShellowSettings
    let renderTick: Int
    let onTerminalInput: (String) -> Void
    let onReconnect: (() -> Void)?
    let onDisconnect: () -> Void
    let onResizeTerminal: (Int, Int) -> Void
    let onAttachRendererSurface: (UInt64, Int, Int) -> Void
    let onSetRendererOverlay: (String) -> Void
    let onRenderRendererSurface: (Int, Int, Int, Int) -> Bool
    let onDetachRendererSurface: () -> Void
    let onClearTerminal: () -> Void
    let onResetTerminal: () -> Void
    @State private var isCtrlArmed = false
    @State private var isAltArmed = false
    @State private var selection: TerminalSelection?
    @State private var isSearchVisible = false
    @State private var searchQuery = ""
    @State private var searchIndex = 0
    @State private var pendingPaste: PendingPaste?
    @State private var pendingRemoteClipboard: PendingRemoteClipboard?
    @State private var transcriptSaveResult: TranscriptSaveResult?
    @State private var handledClipboardSequence = 0
    @State private var keyboardAvoidance = TerminalKeyboardAvoidanceState.hidden

    var body: some View {
        let search = session.searchPresentation(query: searchQuery, focusedIndex: searchIndex)

        GeometryReader { geometry in
            let keyboardInset = keyboardAvoidance.bottomInset(overlapping: geometry.frame(in: .global))
            let bottomSafeInset = geometry.safeAreaInsets.bottom
            let bottomOverlayPadding = keyboardInset > 0 ? keyboardInset : bottomSafeInset
            let bottomChromeHeight = TerminalChromeMetrics.bottomReserve(
                showKeyboardToolbar: settings.showKeyboardToolbar
            )
            let topOverlayPadding = TerminalChromeMetrics.topOverlayPadding(
                safeAreaTop: geometry.safeAreaInsets.top
            )
            let contentTopInset = TerminalChromeMetrics.contentTopInset(
                safeAreaTop: geometry.safeAreaInsets.top,
                showsSearch: isSearchVisible
            )
            let contentBottomInset = bottomChromeHeight + bottomOverlayPadding + TerminalChromeMetrics.contentBottomGap
            let cursorBottomY = session.cursorBottomY(
                fontSize: settings.fontSize,
                lineHeightScale: settings.lineHeightScale,
                viewportHeight: max(1, geometry.size.height - contentTopInset - contentBottomInset),
                topOffset: contentTopInset
            )
            let keyboardCursorOverlap = keyboardAvoidance.cursorAwareOffset(
                overlapping: geometry.frame(in: .global),
                cursorBottomY: cursorBottomY,
                coveredBottomInset: keyboardInset,
                bottomPadding: TerminalChromeMetrics.cursorPadding
            )

            ZStack(alignment: .top) {
                TerminalViewport(
                    session: session,
                    fontSize: settings.fontSize,
                    lineHeightScale: settings.lineHeightScale,
                    search: search,
                    selection: $selection,
                    renderTick: renderTick,
                    contentTopInset: contentTopInset,
                    contentBottomInset: contentBottomInset,
                    reserveBottomScrollSpace: keyboardCursorOverlap > 0,
                    onResizeTerminal: onResizeTerminal,
                    onAttachRendererSurface: onAttachRendererSurface,
                    onSetRendererOverlay: onSetRendererOverlay,
                    onRenderRendererSurface: onRenderRendererSurface,
                    onDetachRendererSurface: onDetachRendererSurface,
                    applicationCursorKeys: session.isApplicationCursorKeysActive,
                    isInputEnabled: !isSearchVisible,
                    sendInput: sendTerminalInput,
                    sendTextInput: handleDirectTextInput,
                    sendBackspace: handleDirectBackspace,
                    handlePaste: handlePaste,
                    copyShortcut: copySelectionOrVisibleTerminal,
                    searchShortcut: showSearch
                )
                .padding(.top, contentTopInset)
                .padding(.bottom, contentBottomInset)

                VStack(spacing: 8) {
                    TerminalFloatingHeader(
                        session: session,
                        onBack: { dismiss() },
                        onReconnect: onReconnect,
                        onDisconnect: onDisconnect
                    )

                    if isSearchVisible {
                        TerminalSearchBar(
                            query: $searchQuery,
                            focusedIndex: $searchIndex,
                            presentation: search,
                            onClose: {
                                isSearchVisible = false
                                searchQuery = ""
                                searchIndex = 0
                            }
                        )
                    }
                }
                .padding(.top, topOverlayPadding)
                .padding(.horizontal, 12)

                VStack {
                    Spacer(minLength: 0)
                    TerminalControlsPanel(
                        isSearchVisible: $isSearchVisible,
                        selectedText: session.selectedText(for: selection),
                        selectedLink: session.selectedText(for: selection)?.firstTerminalURL,
                        showKeyboardToolbar: settings.showKeyboardToolbar,
                        isCtrlArmed: $isCtrlArmed,
                        isAltArmed: $isAltArmed,
                        applicationCursorKeys: session.isApplicationCursorKeysActive,
                        onEnter: sendEnter,
                        onClearTerminal: onClearTerminal,
                        onResetTerminal: onResetTerminal,
                        onSaveTranscript: saveTranscript,
                        onCopyTerminal: copyVisibleTerminal,
                        onCopySelection: copySelection,
                        onCopyLink: copySelectedLink,
                        clearSelection: { selection = nil },
                        onPasteClipboard: pasteFromClipboard,
                        sendInput: sendTerminalInput
                    )
                }
                .padding(.bottom, bottomOverlayPadding)
            }
            .frame(
                width: geometry.size.width,
                height: max(1, geometry.size.height),
                alignment: .top
            )
            .clipped()
            .animation(keyboardAvoidance.animation, value: bottomOverlayPadding)
            .animation(keyboardAvoidance.animation, value: keyboardCursorOverlap)
        }
        .background(ShellowTheme.terminalBackground.ignoresSafeArea())
        .ignoresSafeArea(.container, edges: [.top, .bottom])
        .ignoresSafeArea(.keyboard, edges: .bottom)
        .onAppear {
            presentRemoteClipboardIfNeeded()
        }
        .onReceive(NotificationCenter.default.publisher(for: UIResponder.keyboardWillChangeFrameNotification)) { notification in
            keyboardAvoidance = TerminalKeyboardAvoidanceState(notification: notification)
        }
        .onReceive(NotificationCenter.default.publisher(for: UIResponder.keyboardWillHideNotification)) { notification in
            keyboardAvoidance = TerminalKeyboardAvoidanceState(notification: notification, forceHidden: true)
        }
        .onChange(of: searchQuery) {
            searchIndex = 0
        }
        .onChange(of: session.clipboardSequence) {
            presentRemoteClipboardIfNeeded()
        }
        .alert(item: $pendingPaste) { paste in
            Alert(
                title: Text("Confirm Paste"),
                message: Text(paste.summary),
                primaryButton: .default(Text("Paste")) {
                    commitRiskyPaste(paste.text)
                },
                secondaryButton: .cancel()
            )
        }
        .alert(item: $pendingRemoteClipboard) { request in
            Alert(
                title: Text("Remote Clipboard"),
                message: Text(request.summary),
                primaryButton: .default(Text("Copy")) {
                    UIPasteboard.general.string = request.text
                },
                secondaryButton: .cancel()
            )
        }
        .alert(item: $transcriptSaveResult) { result in
            Alert(
                title: Text(result.title),
                message: Text(result.message),
                dismissButton: .default(Text("OK"))
            )
        }
    }

    private func sendTerminalInput(_ input: String) {
        selection = nil
        onTerminalInput(input)
    }

    private func sendEnter() {
        sendTerminalInput("\r")
        isCtrlArmed = false
        isAltArmed = false
    }

    private func handleDirectTextInput(_ text: String) {
        if isCtrlArmed {
            let encoded = controlEncoded(text)
            if !encoded.isEmpty {
                sendTerminalInput(encoded)
            }
            resetInputModifiers()
            return
        }

        if isAltArmed {
            sendTerminalInput(metaEncoded(text))
            resetInputModifiers()
            return
        }

        let payload = text
            .replacingOccurrences(of: "\r\n", with: "\r")
            .replacingOccurrences(of: "\n", with: "\r")
        guard !payload.isEmpty else { return }
        sendTerminalInput(payload)
    }

    private func handleDirectBackspace() {
        sendTerminalInput("\u{7f}")
        isCtrlArmed = false
        isAltArmed = false
    }

    private func pasteFromClipboard() {
        guard let pasted = UIPasteboard.general.string, !pasted.isEmpty else { return }
        handlePaste(pasted)
    }

    private func handlePaste(_ text: String) {
        if isCtrlArmed {
            sendTerminalInput(controlEncoded(text))
            resetInputModifiers()
            return
        }

        if isAltArmed {
            sendTerminalInput(metaEncoded(text))
            resetInputModifiers()
            return
        }

        if settings.confirmPaste, text.isRiskyTerminalPaste {
            pendingPaste = PendingPaste(text: text)
            return
        }

        if session.isBracketedPasteActive {
            sendPaste(text)
            return
        }

        sendPaste(text)
    }

    private func commitRiskyPaste(_ text: String) {
        sendPaste(text)
    }

    private func sendPaste(_ text: String) {
        if session.isBracketedPasteActive {
            sendTerminalInput("\u{1B}[200~" + text + "\u{1B}[201~")
        } else {
            sendTerminalInput(text)
        }
        isCtrlArmed = false
        isAltArmed = false
    }

    private func copyVisibleTerminal() {
        UIPasteboard.general.string = session.copyableText
    }

    private func saveTranscript() {
        let transcript = session.copyableText
        do {
            let fileURL = try writeTranscript(transcript)
            transcriptSaveResult = TranscriptSaveResult(
                title: "Transcript Saved",
                message: fileURL.lastPathComponent
            )
        } catch {
            transcriptSaveResult = TranscriptSaveResult(
                title: "Save Failed",
                message: error.localizedDescription
            )
        }
    }

    private func writeTranscript(_ transcript: String) throws -> URL {
        let directory = try FileManager.default.url(
            for: .documentDirectory,
            in: .userDomainMask,
            appropriateFor: nil,
            create: true
        )
        .appendingPathComponent("Shellow-Transcripts", isDirectory: true)
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)

        let fileURL = directory.appendingPathComponent(transcriptFileName())
        try transcript.write(to: fileURL, atomically: true, encoding: .utf8)
        return fileURL
    }

    private func transcriptFileName() -> String {
        let formatter = DateFormatter()
        formatter.locale = Locale(identifier: "en_US_POSIX")
        formatter.dateFormat = "yyyyMMdd-HHmmss"
        let timestamp = formatter.string(from: Date())
        let host = session.host.safeTranscriptFileComponent
        return "shellow-\(host)-\(timestamp).txt"
    }

    private func copySelection() {
        guard let selectedText = session.selectedText(for: selection) else { return }
        UIPasteboard.general.string = selectedText
    }

    private func copySelectedLink() {
        guard let link = session.selectedText(for: selection)?.firstTerminalURL else { return }
        UIPasteboard.general.string = link
    }

    private func copySelectionOrVisibleTerminal() {
        UIPasteboard.general.string = session.selectedText(for: selection) ?? session.copyableText
    }

    private func showSearch() {
        isSearchVisible = true
        searchIndex = 0
    }

    private func presentRemoteClipboardIfNeeded() {
        let sequence = session.clipboardSequence
        guard
            sequence > handledClipboardSequence,
            let text = session.pendingClipboardText,
            !text.isEmpty
        else {
            return
        }

        handledClipboardSequence = sequence
        pendingRemoteClipboard = PendingRemoteClipboard(id: sequence, text: text)
    }

    private func resetInputModifiers() {
        isCtrlArmed = false
        isAltArmed = false
    }

    private func controlEncoded(_ text: String) -> String {
        text.compactMap { character in
            guard let scalar = character.unicodeScalars.first else { return nil }
            let lower = CharacterSet.lowercaseLetters.contains(scalar) ? scalar.value : nil
            let upper = CharacterSet.uppercaseLetters.contains(scalar) ? scalar.value : nil
            let value = lower.map { $0 - 96 } ?? upper.map { $0 - 64 }
            guard let value else { return nil }
            return UnicodeScalar(value).map(String.init)
        }
        .joined()
    }

    private func metaEncoded(_ text: String) -> String {
        text.map { "\u{1B}\($0)" }.joined()
    }
}

private struct TerminalFloatingHeader: View {
    let session: TerminalSession
    let onBack: () -> Void
    let onReconnect: (() -> Void)?
    let onDisconnect: () -> Void

    var body: some View {
        HStack(spacing: 8) {
            Button(action: onBack) {
                Image(systemName: "chevron.left")
                    .font(.system(size: 15, weight: .semibold))
                    .frame(width: 34, height: 34)
            }
            .buttonStyle(.plain)
            .foregroundStyle(ShellowTheme.terminalText)
            .background(ShellowTheme.keyBackground.opacity(0.92), in: RoundedRectangle(cornerRadius: 8))
            .accessibilityLabel("Back to Hosts")

            HStack(spacing: 8) {
                Circle()
                    .fill(statusColor)
                    .frame(width: 8, height: 8)

                Text(session.title)
                    .font(.subheadline.weight(.semibold))
                    .lineLimit(1)
                    .minimumScaleFactor(0.82)
            }
            .padding(.horizontal, 12)
            .frame(height: 34)
            .frame(maxWidth: .infinity)
            .background(ShellowTheme.panelBackground.opacity(0.94), in: RoundedRectangle(cornerRadius: 8))
            .overlay(
                RoundedRectangle(cornerRadius: 8)
                    .stroke(ShellowTheme.keyBackground.opacity(0.7), lineWidth: 1)
            )

            if session.state == .disconnected, let onReconnect {
                Button(action: onReconnect) {
                    Image(systemName: "arrow.clockwise")
                        .font(.system(size: 14, weight: .semibold))
                        .frame(width: 34, height: 34)
                }
                .buttonStyle(.plain)
                .foregroundStyle(ShellowTheme.accent)
                .background(ShellowTheme.accent.opacity(0.16), in: RoundedRectangle(cornerRadius: 8))
                .accessibilityLabel("Reconnect Terminal")
            }

            if session.state != .disconnected {
                Button(action: onDisconnect) {
                    Image(systemName: "power")
                        .font(.system(size: 14, weight: .semibold))
                        .frame(width: 34, height: 34)
                }
                .buttonStyle(.plain)
                .foregroundStyle(ShellowTheme.warning)
                .background(ShellowTheme.warning.opacity(0.16), in: RoundedRectangle(cornerRadius: 8))
                .accessibilityLabel("Disconnect Terminal")
            }
        }
        .frame(height: TerminalChromeMetrics.floatingHeaderHeight)
    }

    private var statusColor: Color {
        switch session.state {
        case .connected: ShellowTheme.success
        case .connecting: ShellowTheme.warning
        case .disconnected: ShellowTheme.muted
        }
    }
}

private struct TerminalViewport: View {
    let session: TerminalSession
    let fontSize: Double
    let lineHeightScale: Double
    let search: TerminalSearchPresentation
    @Binding var selection: TerminalSelection?
    let renderTick: Int
    let contentTopInset: CGFloat
    let contentBottomInset: CGFloat
    let reserveBottomScrollSpace: Bool
    let onResizeTerminal: (Int, Int) -> Void
    let onAttachRendererSurface: (UInt64, Int, Int) -> Void
    let onSetRendererOverlay: (String) -> Void
    let onRenderRendererSurface: (Int, Int, Int, Int) -> Bool
    let onDetachRendererSurface: () -> Void
    let applicationCursorKeys: Bool
    let isInputEnabled: Bool
    let sendInput: (String) -> Void
    let sendTextInput: (String) -> Void
    let sendBackspace: () -> Void
    let handlePaste: (String) -> Void
    let copyShortcut: () -> Void
    let searchShortcut: () -> Void
    @State private var lastReportedGrid: TerminalGridSize?
    @State private var inputFocusNonce = 0
    @State private var gridScrollOffsetY: CGFloat = 0
    @State private var gridUsesDirectScrollGeometry = false

    var body: some View {
        GeometryReader { geometry in
            Group {
                if let grid = session.grid, grid.hasVisibleContent || grid.activeScreen == .alternate {
                    ScrollViewReader { proxy in
                        ZStack(alignment: .bottomTrailing) {
                            ScrollView(.vertical) {
                                VStack(alignment: .leading, spacing: 0) {
                                    TerminalGridSurface(
                                        grid: grid,
                                        fontSize: fontSize,
                                        lineHeightScale: lineHeightScale,
                                        renderTick: renderTick,
                                        scrollOffsetY: gridUsesDirectScrollGeometry ? gridScrollOffsetY : nil,
                                        search: search,
                                        selection: $selection,
                                        onAttachRendererSurface: onAttachRendererSurface,
                                        onSetRendererOverlay: onSetRendererOverlay,
                                        onRenderRendererSurface: onRenderRendererSurface,
                                        onDetachRendererSurface: onDetachRendererSurface,
                                        sendInput: sendInput
                                    )
                                    .padding(.horizontal, TerminalMetrics.viewportHorizontalPadding)
                                    .padding(.vertical, TerminalMetrics.viewportVerticalPadding)

                                    Color.clear
                                        .frame(height: 1)
                                        .id(Self.bottomAnchorID)
                                }
                            }
                            .scrollIndicators(.hidden)
                            .coordinateSpace(name: TerminalGridSurface.scrollCoordinateSpaceName)
                            .trackingGridScrollOffset { offsetY in
                                gridUsesDirectScrollGeometry = true
                                gridScrollOffsetY = offsetY
                            }
                            .id(grid.activeScreen)

                            if !grid.lines.isEmpty, grid.activeScreen == .primary {
                                TerminalJumpToBottomButton {
                                    scrollGridToBottom(proxy, lineCount: grid.lines.count)
                                }
                                .padding(14)
                            }
                        }
                        .onAppear {
                            guard search.isEmpty else { return }
                            if grid.activeScreen == .alternate {
                                scrollGridToTop(proxy, lineCount: grid.lines.count)
                            } else {
                                scrollGridToBottom(proxy, lineCount: grid.lines.count, animated: false)
                            }
                        }
                        .onChange(of: grid.activeScreen) {
                            guard search.isEmpty, grid.activeScreen == .alternate else { return }
                            scrollGridToTop(proxy, lineCount: grid.lines.count)
                        }
                        .onChange(of: search.activeHit) {
                            guard let row = search.activeGridRow else { return }
                            withAnimation(.snappy(duration: 0.18)) {
                                proxy.scrollTo(TerminalSearchHit.gridRowID(row), anchor: .center)
                            }
                        }
                        .onChange(of: grid.lines.count) {
                            guard search.isEmpty, !grid.lines.isEmpty, grid.activeScreen == .primary else { return }
                            scrollGridToBottom(proxy, lineCount: grid.lines.count)
                        }
                        .onChange(of: reserveBottomScrollSpace) {
                            guard search.isEmpty, !grid.lines.isEmpty, grid.activeScreen == .primary else { return }
                            scrollGridToBottom(proxy, lineCount: grid.lines.count)
                        }
                    }
                } else {
                    ScrollViewReader { proxy in
                        ZStack(alignment: .bottomTrailing) {
                            ScrollView {
                                LazyVStack(alignment: .leading, spacing: 6) {
                                    ForEach(Array(session.rows.enumerated()), id: \.element.id) { index, row in
                                        TerminalRowView(
                                            row: row,
                                            fontSize: fontSize,
                                            isSelected: selection?.containsHistory(row: index) == true,
                                            isSearchMatch: search.containsHistory(row: index),
                                            isActiveSearchMatch: search.activeHit == .history(index)
                                        )
                                        .contentShape(Rectangle())
                                        .onLongPressGesture(minimumDuration: 0.35) {
                                            selection = selection?.extendingHistory(to: index) ?? .history(anchor: index, focus: index)
                                        }
                                        .accessibilityLabel("Long press to select terminal row \(index + 1)")
                                        .id(TerminalSearchHit.history(index).id)
                                    }

                                    Color.clear
                                        .frame(height: 1)
                                        .id(Self.bottomAnchorID)
                                }
                                .padding(.horizontal, TerminalMetrics.viewportHorizontalPadding)
                                .padding(.vertical, TerminalMetrics.viewportVerticalPadding)
                                .frame(maxWidth: .infinity, alignment: .leading)
                                .id(historyRenderKey)
                            }
                            .scrollIndicators(.hidden)

                            if !session.rows.isEmpty {
                                TerminalJumpToBottomButton {
                                    scrollHistoryToBottom(proxy, rowCount: session.rows.count)
                                }
                                .padding(14)
                            }
                        }
                        .onAppear {
                            guard search.isEmpty else { return }
                            scrollHistoryToBottom(proxy, rowCount: session.rows.count, animated: false)
                        }
                        .onChange(of: session.rows.count) {
                            guard search.isEmpty else { return }
                            scrollHistoryToBottom(proxy, rowCount: session.rows.count)
                        }
                        .onChange(of: reserveBottomScrollSpace) {
                            guard search.isEmpty else { return }
                            scrollHistoryToBottom(proxy, rowCount: session.rows.count)
                        }
                        .onChange(of: search.activeHit) {
                            guard case .history(let row)? = search.activeHit else { return }
                            withAnimation(.snappy(duration: 0.18)) {
                                proxy.scrollTo(TerminalSearchHit.history(row).id, anchor: .center)
                            }
                        }
                    }
                }
            }
            .contentShape(Rectangle())
            .simultaneousGesture(
                TapGesture().onEnded {
                    guard isInputEnabled else { return }
                    inputFocusNonce += 1
                }
            )
            .overlay(alignment: .topLeading) {
                TerminalDirectInputCapture(
                    applicationCursorKeys: applicationCursorKeys,
                    isEnabled: isInputEnabled,
                    focusNonce: inputFocusNonce,
                    sendInput: sendInput,
                    sendTextInput: sendTextInput,
                    sendBackspace: sendBackspace,
                    handlePaste: handlePaste,
                    copyShortcut: copyShortcut,
                    searchShortcut: searchShortcut
                )
                .frame(width: 1, height: 1)
                .opacity(0.01)
                .allowsHitTesting(false)
                .accessibilityHidden(true)
            }
            .onAppear { report(size: geometry.size) }
            .onAppear {
                guard isInputEnabled else { return }
                inputFocusNonce += 1
            }
            .onChange(of: isInputEnabled) {
                guard isInputEnabled else { return }
                inputFocusNonce += 1
            }
            .onChange(of: geometry.size) { report(size: geometry.size) }
            .onChange(of: contentTopInset) { report(size: geometry.size) }
            .onChange(of: contentBottomInset) { report(size: geometry.size) }
            .onChange(of: fontSize) { report(size: geometry.size) }
            .onChange(of: lineHeightScale) { report(size: geometry.size) }
        }
    }

    private static let bottomAnchorID = "terminal-bottom-anchor"

    private func report(size: CGSize) {
        let grid = TerminalGridSize(size: size, fontSize: fontSize, lineHeightScale: lineHeightScale)
        guard grid != lastReportedGrid else { return }
        lastReportedGrid = grid
        onResizeTerminal(grid.cols, grid.rows)
    }

    private func scrollGridToBottom(_ proxy: ScrollViewProxy, lineCount: Int, animated: Bool = true) {
        guard lineCount > 0 else { return }
        if animated {
            withAnimation(.snappy(duration: 0.18)) {
                proxy.scrollTo(Self.bottomAnchorID, anchor: .bottomLeading)
            }
        } else {
            proxy.scrollTo(Self.bottomAnchorID, anchor: .bottomLeading)
        }
    }

    private func scrollGridToTop(_ proxy: ScrollViewProxy, lineCount: Int) {
        guard lineCount > 0 else { return }
        DispatchQueue.main.async {
            proxy.scrollTo(TerminalSearchHit.gridRowID(0), anchor: .topLeading)
        }
    }

    private func scrollHistoryToBottom(_ proxy: ScrollViewProxy, rowCount: Int, animated: Bool = true) {
        guard rowCount > 0 else { return }
        if animated {
            withAnimation(.snappy(duration: 0.18)) {
                proxy.scrollTo(Self.bottomAnchorID, anchor: .bottom)
            }
        } else {
            proxy.scrollTo(Self.bottomAnchorID, anchor: .bottom)
        }
    }

    private var historyRenderKey: String {
        let first = session.rows.first.map(rowRenderKey) ?? "empty"
        let last = session.rows.last.map(rowRenderKey) ?? "empty"
        return "\(session.rows.count)|\(first)|\(last)"
    }

    private func rowRenderKey(_ row: TerminalRow) -> String {
        "\(row.style.rawValue)|\(row.prompt)|\(row.text)"
    }
}

private struct TerminalGridSurface: View {
    static let scrollCoordinateSpaceName = "TerminalGridScroll"

    let grid: TerminalGridSnapshot
    let fontSize: Double
    let lineHeightScale: Double
    let renderTick: Int
    let scrollOffsetY: CGFloat?
    let search: TerminalSearchPresentation
    @Binding var selection: TerminalSelection?
    let onAttachRendererSurface: (UInt64, Int, Int) -> Void
    let onSetRendererOverlay: (String) -> Void
    let onRenderRendererSurface: (Int, Int, Int, Int) -> Bool
    let onDetachRendererSurface: () -> Void
    let sendInput: (String) -> Void
    @State private var activeSelectionDragAnchor: TerminalSelectionPoint?
    @State private var suppressNextMouseTap = false
    @State private var contentMinY = TerminalMetrics.viewportVerticalPadding

    var body: some View {
        let currentViewportFirstRow = viewportFirstRow
        let currentViewportRowCount = viewportRowCount

        ZStack(alignment: .topLeading) {
            if TerminalMetalGridSurface.isAvailable {
                TerminalMetalGridSurface(
                    viewportFirstRow: currentViewportFirstRow,
                    viewportRowCount: currentViewportRowCount,
                    renderTick: renderTick,
                    rendererOverlayJSON: rendererOverlayJSON,
                    attachRendererSurface: onAttachRendererSurface,
                    setRendererOverlay: onSetRendererOverlay,
                    renderRendererSurface: onRenderRendererSurface,
                    detachRendererSurface: onDetachRendererSurface
                )
                .frame(width: metalSurfaceWidth, height: viewportSurfaceHeight, alignment: .topLeading)
                .offset(y: scrollOffset)
                .allowsHitTesting(false)
                .accessibilityIdentifier("Metal Terminal Grid")
            }

            VStack(alignment: .leading, spacing: 0) {
                ForEach(Array(grid.lines.enumerated()), id: \.offset) { row, _ in
                    Button {
                        if suppressNextMouseTap {
                            suppressNextMouseTap = false
                            return
                        }
                        if let mouseInput = grid.mousePressSequence(row: row, column: 0) {
                            selection = nil
                            sendInput(mouseInput)
                        }
                    } label: {
                        Color.clear
                            .frame(width: metalSurfaceWidth, height: rowHeight, alignment: .leading)
                            .contentShape(Rectangle())
                    }
                    .buttonStyle(.plain)
                    .simultaneousGesture(
                        LongPressGesture(minimumDuration: 0.35)
                            .sequenced(before: DragGesture(minimumDistance: 0))
                            .onChanged { value in
                                switch value {
                                case .first(true):
                                    selection = .gridRow(row)
                                    suppressNextMouseTap = true
                                case .second(true, let drag?):
                                    let start = activeSelectionDragAnchor
                                        ?? selectionPoint(from: drag.startLocation, initialRow: row)
                                    activeSelectionDragAnchor = start
                                    let focus = selectionPoint(from: drag.location, initialRow: row)
                                    selection = .grid(anchor: start, focus: focus)
                                    suppressNextMouseTap = true
                                default:
                                    break
                                }
                            }
                            .onEnded { value in
                                if case .first(true) = value {
                                    selection = .gridRow(row)
                                }
                                suppressNextMouseTap = true
                                activeSelectionDragAnchor = nil
                            },
                        including: .all
                    )
                    .accessibilityLabel(grid.mouseReporting ? "Send mouse event row \(row + 1), long press to select" : "Long press to select terminal grid row \(row + 1)")
                    .id(TerminalSearchHit.gridRowID(row))
                }
            }
            .frame(width: metalSurfaceWidth, height: fullContentHeight, alignment: .topLeading)
            .background(
                GeometryReader { proxy in
                    Color.clear.preference(
                        key: TerminalGridContentMinYPreferenceKey.self,
                        value: proxy.frame(in: .named(Self.scrollCoordinateSpaceName)).minY
                    )
                }
            )
        }
        .frame(
            minWidth: metalSurfaceWidth,
            minHeight: fullContentHeight,
            alignment: .leading
        )
        .onPreferenceChange(TerminalGridContentMinYPreferenceKey.self) { value in
            contentMinY = value
        }
    }

    private var metalSurfaceWidth: CGFloat {
        CGFloat(grid.cols) * TerminalMetrics.cellWidth(fontSize: fontSize)
            + TerminalMetrics.horizontalInset * 2
    }

    private var viewportSurfaceHeight: CGFloat {
        CGFloat(viewportRowCount) * rowHeight
    }

    private var fullContentHeight: CGFloat {
        CGFloat(max(1, max(grid.lines.count, Int(grid.rows)))) * rowHeight
    }

    private var viewportRowCount: Int {
        max(1, Int(grid.rows))
    }

    private var viewportFirstRow: Int {
        grid.viewportFirstRow(scrollOffset: scrollOffset, rowHeight: rowHeight)
    }

    private var visibleGridRowRange: Range<Int> {
        let first = viewportFirstRow
        return first..<min(grid.lines.count, first + viewportRowCount)
    }

    private var scrollOffset: CGFloat {
        scrollOffsetY ?? max(0, TerminalMetrics.viewportVerticalPadding - contentMinY)
    }

    private var selectedGridCellRanges: [Int: Range<Int>] {
        Dictionary(
            uniqueKeysWithValues: grid.lines.indices.compactMap { row in
                let line = grid.lines[row]
                guard visibleGridRowRange.contains(row) else { return nil }
                guard let range = selection?.gridCellRange(row: row, line: line) else { return nil }
                return (row - viewportFirstRow, range)
            }
        )
    }

    private var fullSelectedGridRows: Set<Int> {
        Set(
            grid.lines.indices.filter { row in
                visibleGridRowRange.contains(row) &&
                selection?.isFullGridRow(row: row, line: grid.lines[row]) == true
            }
            .map { row in
                row - viewportFirstRow
            }
        )
    }

    private var searchGridRows: Set<Int> {
        Set(
            search.hits.compactMap(\.gridRow).compactMap { row in
                visibleGridRowRange.contains(row) ? row - viewportFirstRow : nil
            }
        )
    }

    private var activeSearchGridRow: Int? {
        guard let row = search.activeGridRow, visibleGridRowRange.contains(row) else { return nil }
        return row - viewportFirstRow
    }

    private var rendererOverlayJSON: String {
        var ranges: [[String: Any]] = []
        for (row, range) in selectedGridCellRanges.sorted(by: { $0.key < $1.key }) {
            ranges.append(rendererOverlayRange(kind: "selection", row: row, start: range.lowerBound, end: range.upperBound))
        }
        for globalRow in search.hits.compactMap(\.gridRow).filter({ visibleGridRowRange.contains($0) }).sorted() {
            let viewportRow = globalRow - viewportFirstRow
            for range in search.gridRanges(row: globalRow) {
                ranges.append(rendererOverlayRange(kind: "search", row: viewportRow, start: range.lowerBound, end: range.upperBound))
            }
        }
        if let row = activeSearchGridRow, let range = search.activeGridRange {
            ranges.append(rendererOverlayRange(kind: "active_search", row: row, start: range.lowerBound, end: range.upperBound))
        }

        guard
            let data = try? JSONSerialization.data(withJSONObject: ["ranges": ranges], options: []),
            let json = String(data: data, encoding: .utf8)
        else {
            return #"{"ranges":[]}"#
        }
        return json
    }

    private func rendererOverlayRange(kind: String, row: Int, start: Int, end: Int) -> [String: Any] {
        [
            "kind": kind,
            "row": max(row, 0),
            "start_col": max(start, 0),
            "end_col": max(end, 0)
        ]
    }

    private var cellWidth: CGFloat {
        TerminalMetrics.cellWidth(fontSize: fontSize)
    }

    private var rowHeight: CGFloat {
        TerminalMetrics.rowHeight(fontSize: fontSize, lineHeightScale: lineHeightScale)
    }

    private func selectionPoint(from location: CGPoint, initialRow: Int) -> TerminalSelectionPoint {
        let rowOffset = Int(floor(location.y / rowHeight))
        let projectedRow = max(0, min(initialRow + rowOffset, max(0, grid.lines.count - 1)))
        let line = grid.lines.indices.contains(projectedRow) ? grid.lines[projectedRow] : ""
        let projectedColumn = Int(floor((location.x - TerminalMetrics.horizontalInset) / cellWidth))
        let column = max(0, min(projectedColumn, line.terminalCellWidth))
        return TerminalSelectionPoint(row: projectedRow, column: column)
    }

    private func rowBackground(row: Int, isFullySelected: Bool) -> Color {
        if isFullySelected {
            return ShellowTheme.selectionBackground
        }
        if search.activeGridRow == row {
            return ShellowTheme.searchCurrentBackground
        }
        if search.containsGrid(row: row) {
            return ShellowTheme.searchBackground
        }
        return .clear
    }

    private func attributedLine(row: Int, selection: Range<Int>?) -> AttributedString {
        let line = grid.lines.indices.contains(row) ? grid.lines[row] : ""
        let sourceRuns = grid.styledLines.indices.contains(row) && !grid.styledLines[row].runs.isEmpty
            ? grid.styledLines[row].runs
            : [TerminalGridRun(text: line.isEmpty ? " " : line, style: .plain)]

        var result = AttributedString()
        let cursorOffset = grid.cursorVisible && row == grid.cursorRow
            ? max(0, min(grid.cursorColumn, max(0, grid.cols - 1)))
            : nil
        let cursorGlyph = grid.cursorShape.glyph
        var consumed = 0
        var cursorWasWritten = false

        for run in sourceRuns {
            let insertion = run.text.withTerminalCursor(
                targetColumn: cursorWasWritten ? nil : cursorOffset,
                consumedCells: consumed,
                glyph: cursorGlyph
            )
            append(insertion.text, style: run.style, selection: selection, consumedCells: consumed, to: &result)
            consumed = insertion.nextCellColumn
            cursorWasWritten = cursorWasWritten || insertion.didWrite
        }

        if let cursorOffset, !cursorWasWritten {
            if cursorOffset > consumed {
                append(
                    String(repeating: " ", count: cursorOffset - consumed),
                    style: .plain,
                    selection: selection,
                    consumedCells: consumed,
                    to: &result
                )
                consumed = cursorOffset
            }
            append(String(cursorGlyph), style: .plain, selection: selection, consumedCells: consumed, to: &result)
            cursorWasWritten = true
        }

        if result.characters.isEmpty {
            append(" ", style: .plain, to: &result)
        }

        return result
    }

    private func append(
        _ text: String,
        style: TerminalGridStyle,
        selection: Range<Int>? = nil,
        consumedCells: Int = 0,
        to result: inout AttributedString
    ) {
        guard !text.isEmpty else { return }
        guard let selection else {
            appendRun(text, style: style, isSelected: false, to: &result)
            return
        }

        var cell = consumedCells
        for character in text {
            let width = character.terminalCellWidth
            let selected = width > 0 && selection.overlaps(cell..<(cell + width))
            appendRun(String(character), style: style, isSelected: selected, to: &result)
            cell += width
        }
    }

    private func appendRun(
        _ text: String,
        style: TerminalGridStyle,
        isSelected: Bool,
        to result: inout AttributedString
    ) {
        guard !text.isEmpty else { return }
        var run = AttributedString(text)
        let foreground = resolvedForeground(for: style)
        run.foregroundColor = foreground
        if isSelected {
            run.backgroundColor = ShellowTheme.selectionBackground
        } else if let background = resolvedBackground(for: style) {
            run.backgroundColor = background
        }
        if style.underline {
            run.underlineStyle = .single
        }
        if style.strikethrough {
            run.strikethroughStyle = .single
        }
        run.font = .system(
            size: fontSize,
            weight: style.bold ? .semibold : .regular,
            design: .monospaced
        )
        result.append(run)
    }

    private func resolvedForeground(for style: TerminalGridStyle) -> Color {
        if style.inverse {
            return style.bg?.color ?? ShellowTheme.terminalBackground
        }
        return style.fg?.color ?? ShellowTheme.terminalText
    }

    private func resolvedBackground(for style: TerminalGridStyle) -> Color? {
        if style.inverse {
            return style.fg?.color ?? ShellowTheme.terminalText
        }
        return style.bg?.color
    }
}

private extension TerminalGridSnapshot {
    func viewportFirstRow(scrollOffset: CGFloat, rowHeight: CGFloat) -> Int {
        guard activeScreen == .primary, rowHeight > 0, lines.count > rows else { return 0 }
        let visibleRows = max(1, Int(rows))
        let maxFirstRow = max(0, lines.count - visibleRows)
        let requestedFirstRow = Int((scrollOffset / rowHeight).rounded(.down))
        return max(0, min(requestedFirstRow, maxFirstRow))
    }
}

private struct TerminalGridContentMinYPreferenceKey: PreferenceKey {
    static let defaultValue: CGFloat = TerminalMetrics.viewportVerticalPadding

    static func reduce(value: inout CGFloat, nextValue: () -> CGFloat) {
        value = nextValue()
    }
}

private extension View {
    @ViewBuilder
    func trackingGridScrollOffset(_ onChange: @escaping (CGFloat) -> Void) -> some View {
        if #available(iOS 18.0, *) {
            self.onScrollGeometryChange(for: CGFloat.self) { geometry in
                max(0, geometry.contentOffset.y + geometry.contentInsets.top)
            } action: { _, newValue in
                onChange(newValue)
            }
        } else {
            self
        }
    }
}

private struct TerminalCursorInsertion {
    let text: String
    let nextCellColumn: Int
    let didWrite: Bool
}

private extension String {
    var firstTerminalURL: String? {
        guard
            let detector = try? NSDataDetector(types: NSTextCheckingResult.CheckingType.link.rawValue)
        else {
            return nil
        }

        let range = NSRange(startIndex..<endIndex, in: self)
        return detector
            .firstMatch(in: self, options: [], range: range)?
            .url?
            .absoluteString
    }

    func withTerminalCursor(
        targetColumn: Int?,
        consumedCells: Int,
        glyph: Character
    ) -> TerminalCursorInsertion {
        guard let targetColumn else {
            return TerminalCursorInsertion(
                text: self,
                nextCellColumn: consumedCells + terminalCellWidth,
                didWrite: false
            )
        }

        var output = ""
        var cell = consumedCells
        var didWrite = false

        for character in self {
            let width = character.terminalCellWidth
            if !didWrite, width > 0, targetColumn >= cell, targetColumn < cell + width {
                output.append(glyph)
                if width > 1 {
                    output += String(repeating: " ", count: width - 1)
                }
                didWrite = true
            } else {
                output.append(character)
            }
            cell += width
        }

        return TerminalCursorInsertion(text: output, nextCellColumn: cell, didWrite: didWrite)
    }

    var terminalCellWidth: Int {
        reduce(0) { $0 + $1.terminalCellWidth }
    }

    func terminalSearchCellRanges(query: String) -> [Range<Int>] {
        guard !query.isEmpty else { return [] }

        var ranges: [Range<Int>] = []
        var searchStart = startIndex
        while searchStart < endIndex,
              let match = range(of: query, options: [.caseInsensitive], range: searchStart..<endIndex) {
            if let cellRange = terminalCellRange(for: match) {
                ranges.append(cellRange)
            }
            searchStart = match.upperBound
            if searchStart == match.lowerBound, searchStart < endIndex {
                formIndex(after: &searchStart)
            }
        }
        return ranges
    }

    private func terminalCellRange(for characterRange: Range<String.Index>) -> Range<Int>? {
        var index = startIndex
        var cell = 0
        var firstCell: Int?
        var lastCell = 0

        while index < endIndex {
            let next = self.index(after: index)
            let width = self[index].terminalCellWidth
            if width > 0, index < characterRange.upperBound, characterRange.lowerBound < next {
                if firstCell == nil {
                    firstCell = cell
                }
                lastCell = cell + width
            }
            cell += width
            index = next
        }

        guard let start = firstCell else { return nil }
        return start..<max(lastCell, start + 1)
    }

    func terminalSubstring(cells range: Range<Int>) -> String {
        guard !range.isEmpty else { return "" }
        var output = ""
        var cell = 0

        for character in self {
            let width = character.terminalCellWidth
            if width > 0, range.overlaps(cell..<(cell + width)) {
                output.append(character)
            }
            cell += width
        }

        return output
    }
}

private extension Character {
    var terminalCellWidth: Int {
        if unicodeScalars.allSatisfy(\.isZeroWidthTerminalScalar) {
            return 0
        }
        if unicodeScalars.contains(where: \.isWideTerminalScalar) {
            return 2
        }
        return 1
    }
}

private extension Unicode.Scalar {
    var isZeroWidthTerminalScalar: Bool {
        switch value {
        case 0x0300...0x036F,
             0x1AB0...0x1AFF,
             0x1DC0...0x1DFF,
             0x20D0...0x20FF,
             0xFE00...0xFE0F,
             0xE0100...0xE01EF,
             0x200D:
            return true
        default:
            return false
        }
    }

    var isWideTerminalScalar: Bool {
        switch value {
        case 0x1100...0x115F,
             0x2329...0x232A,
             0x2E80...0xA4CF,
             0xAC00...0xD7A3,
             0xF900...0xFAFF,
             0xFE10...0xFE19,
             0xFE30...0xFE6F,
             0xFF00...0xFF60,
             0xFFE0...0xFFE6,
             0x1F000...0x1FAFF,
             0x20000...0x3FFFD:
            return true
        default:
            return false
        }
    }
}

private extension TerminalCursorShape {
    var glyph: Character {
        switch self {
        case .block: "\u{2588}"
        case .underline: "\u{2581}"
        case .bar: "\u{258F}"
        }
    }
}

private struct TerminalRowView: View {
    let row: TerminalRow
    let fontSize: Double
    var isSelected = false
    var isSearchMatch = false
    var isActiveSearchMatch = false

    var body: some View {
        HStack(alignment: .firstTextBaseline, spacing: 8) {
            Text(row.prompt)
                .foregroundStyle(ShellowTheme.prompt)
                .frame(width: 14, alignment: .trailing)

            Text(row.text.isEmpty ? " " : row.text)
                .foregroundStyle(foreground)
                .lineLimit(nil)
                .textSelection(.enabled)

            if row.style == .prompt {
                Rectangle()
                    .fill(ShellowTheme.accent)
                    .frame(width: 8, height: fontSize + 3)
                    .opacity(0.9)
            }
        }
        .font(.system(size: fontSize, weight: .regular, design: .monospaced))
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.horizontal, 3)
        .padding(.vertical, 1)
        .background(background)
    }

    private var foreground: Color {
        switch row.style {
        case .command, .prompt: ShellowTheme.terminalText
        case .muted: ShellowTheme.terminalMuted
        case .success: ShellowTheme.success
        case .warning: ShellowTheme.warning
        }
    }

    private var background: Color {
        if isSelected {
            return ShellowTheme.selectionBackground
        }
        if isActiveSearchMatch {
            return ShellowTheme.searchCurrentBackground
        }
        if isSearchMatch {
            return ShellowTheme.searchBackground
        }
        return .clear
    }
}

private struct TerminalSearchBar: View {
    @Binding var query: String
    @Binding var focusedIndex: Int
    let presentation: TerminalSearchPresentation
    let onClose: () -> Void

    var body: some View {
        HStack(spacing: 8) {
            Image(systemName: "magnifyingglass")
                .font(.system(size: 14, weight: .semibold))
                .foregroundStyle(ShellowTheme.terminalMuted)

            TextField("Search", text: $query)
                .textInputAutocapitalization(.never)
                .autocorrectionDisabled()
                .font(.system(size: 14, design: .monospaced))
                .padding(.horizontal, 10)
                .padding(.vertical, 8)
                .background(ShellowTheme.inputBackground, in: RoundedRectangle(cornerRadius: 8))

            Text(matchText)
                .font(.caption.weight(.medium))
                .foregroundStyle(ShellowTheme.terminalMuted)
                .frame(width: 52, alignment: .trailing)

            Button(action: previous) {
                Image(systemName: "chevron.up")
                    .font(.system(size: 13, weight: .semibold))
                    .frame(width: 32, height: 32)
            }
            .buttonStyle(.plain)
            .foregroundStyle(ShellowTheme.terminalText)
            .background(ShellowTheme.keyBackground, in: RoundedRectangle(cornerRadius: 8))
            .accessibilityLabel("Previous Search Match")

            Button(action: next) {
                Image(systemName: "chevron.down")
                    .font(.system(size: 13, weight: .semibold))
                    .frame(width: 32, height: 32)
            }
            .buttonStyle(.plain)
            .foregroundStyle(ShellowTheme.terminalText)
            .background(ShellowTheme.keyBackground, in: RoundedRectangle(cornerRadius: 8))
            .accessibilityLabel("Next Search Match")

            Button(action: onClose) {
                Image(systemName: "xmark")
                    .font(.system(size: 13, weight: .semibold))
                    .frame(width: 32, height: 32)
            }
            .buttonStyle(.plain)
            .foregroundStyle(ShellowTheme.terminalText)
            .background(ShellowTheme.keyBackground, in: RoundedRectangle(cornerRadius: 8))
            .accessibilityLabel("Close Search")
        }
        .padding(8)
        .background(ShellowTheme.panelBackground.opacity(0.96), in: RoundedRectangle(cornerRadius: 8))
        .overlay(
            RoundedRectangle(cornerRadius: 8)
                .stroke(ShellowTheme.keyBackground.opacity(0.7), lineWidth: 1)
        )
    }

    private var matchText: String {
        guard !presentation.isEmpty else { return "" }
        guard !presentation.hits.isEmpty else { return "0/0" }
        return "\(presentation.activeOrdinal)/\(presentation.hits.count)"
    }

    private func previous() {
        let count = presentation.hits.count
        guard count > 0 else { return }
        focusedIndex = (focusedIndex - 1 + count) % count
    }

    private func next() {
        let count = presentation.hits.count
        guard count > 0 else { return }
        focusedIndex = (focusedIndex + 1) % count
    }
}

private struct TerminalGridSize: Equatable {
    var cols: Int
    var rows: Int

    init(size: CGSize, fontSize: Double, lineHeightScale: Double) {
        let charWidth = TerminalMetrics.cellWidth(fontSize: fontSize)
        let rowHeight = TerminalMetrics.rowHeight(
            fontSize: fontSize,
            lineHeightScale: lineHeightScale
        )
        let availableWidth = max(1, size.width - TerminalMetrics.viewportHorizontalPadding * 2)
        let availableHeight = max(1, size.height - TerminalMetrics.viewportVerticalPadding * 2)
        cols = max(TerminalMetrics.minimumPortraitColumns, min(300, Int(availableWidth / charWidth)))
        rows = max(TerminalMetrics.minimumRows, min(120, Int(availableHeight / rowHeight)))
    }
}

enum TerminalMetrics {
    static let minimumPortraitColumns = 32
    static let minimumRows = 6
    static let horizontalInset: CGFloat = 0
    static let viewportHorizontalPadding: CGFloat = 8
    static let viewportVerticalPadding: CGFloat = 8

    static func cellWidth(fontSize: Double) -> CGFloat {
        max(6.5, CGFloat(fontSize) * 0.56)
    }

    static func rowHeight(fontSize: Double, lineHeightScale: Double) -> CGFloat {
        (max(14.0, CGFloat(fontSize) * 1.25) + 3.0) * CGFloat(lineHeightScale)
    }
}

private enum TerminalChromeMetrics {
    static let floatingHeaderHeight: CGFloat = 42
    static let floatingHeaderTopPadding: CGFloat = 8
    static let cursorPadding: CGFloat = 18
    static let searchHeight: CGFloat = 48
    static let headerContentGap: CGFloat = 8
    static let contentBottomGap: CGFloat = 8
    static let fallbackSafeAreaTop: CGFloat = 52

    static func topOverlayPadding(safeAreaTop: CGFloat) -> CGFloat {
        max(safeAreaTop, fallbackSafeAreaTop) + floatingHeaderTopPadding
    }

    static func contentTopInset(safeAreaTop: CGFloat, showsSearch: Bool) -> CGFloat {
        topOverlayPadding(safeAreaTop: safeAreaTop)
            + floatingHeaderHeight
            + headerContentGap
            + (showsSearch ? searchHeight + headerContentGap : 0)
    }

    static func bottomReserve(showKeyboardToolbar: Bool) -> CGFloat {
        showKeyboardToolbar ? 104 : 58
    }
}

private struct TerminalControlsPanel: View {
    @Binding var isSearchVisible: Bool
    let selectedText: String?
    let selectedLink: String?
    let showKeyboardToolbar: Bool
    @Binding var isCtrlArmed: Bool
    @Binding var isAltArmed: Bool
    let applicationCursorKeys: Bool
    let onEnter: () -> Void
    let onClearTerminal: () -> Void
    let onResetTerminal: () -> Void
    let onSaveTranscript: () -> Void
    let onCopyTerminal: () -> Void
    let onCopySelection: () -> Void
    let onCopyLink: () -> Void
    let clearSelection: () -> Void
    let onPasteClipboard: () -> Void
    let sendInput: (String) -> Void

    var body: some View {
        VStack(spacing: 8) {
            TerminalInputBar(
                isSearchVisible: $isSearchVisible,
                selectedText: selectedText,
                selectedLink: selectedLink,
                onEnter: onEnter,
                onClearTerminal: onClearTerminal,
                onResetTerminal: onResetTerminal,
                onSaveTranscript: onSaveTranscript,
                onCopyTerminal: onCopyTerminal,
                onCopySelection: onCopySelection,
                onCopyLink: onCopyLink,
                clearSelection: clearSelection,
                onPasteClipboard: onPasteClipboard
            )

            if showKeyboardToolbar {
                TerminalKeyboardToolbar(
                    isCtrlArmed: $isCtrlArmed,
                    isAltArmed: $isAltArmed,
                    applicationCursorKeys: applicationCursorKeys,
                    sendInput: sendInput
                )
            }
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 10)
        .background(ShellowTheme.panelBackground.opacity(0.97))
        .overlay(alignment: .top) {
            Rectangle()
                .fill(ShellowTheme.keyBackground.opacity(0.85))
                .frame(height: 1)
        }
    }
}

private struct TerminalInputBar: View {
    @Binding var isSearchVisible: Bool
    let selectedText: String?
    let selectedLink: String?
    let onEnter: () -> Void
    let onClearTerminal: () -> Void
    let onResetTerminal: () -> Void
    let onSaveTranscript: () -> Void
    let onCopyTerminal: () -> Void
    let onCopySelection: () -> Void
    let onCopyLink: () -> Void
    let clearSelection: () -> Void
    let onPasteClipboard: () -> Void

    var body: some View {
        HStack(spacing: 10) {
            ScrollView(.horizontal) {
                HStack(spacing: 10) {
                    TerminalIconButton(
                        systemName: "trash",
                        accessibilityLabel: "Clear Terminal",
                        action: onClearTerminal
                    )
                    TerminalIconButton(
                        systemName: "arrow.counterclockwise",
                        accessibilityLabel: "Reset Terminal",
                        action: onResetTerminal
                    )
                    TerminalIconButton(
                        systemName: "square.and.arrow.down",
                        accessibilityLabel: "Save Transcript",
                        action: onSaveTranscript
                    )
                    TerminalIconButton(
                        systemName: "doc.on.doc",
                        accessibilityLabel: "Copy Terminal",
                        action: onCopyTerminal
                    )
                    TerminalIconButton(
                        systemName: "magnifyingglass",
                        accessibilityLabel: "Search Terminal",
                        foreground: isSearchVisible ? ShellowTheme.accent : ShellowTheme.terminalText
                    ) {
                        isSearchVisible.toggle()
                    }

                    if selectedText != nil {
                        TerminalIconButton(
                            systemName: "doc.on.doc.fill",
                            accessibilityLabel: "Copy Selection",
                            foreground: ShellowTheme.accent,
                            action: onCopySelection
                        )

                        if selectedLink != nil {
                            TerminalIconButton(
                                systemName: "link",
                                accessibilityLabel: "Copy Link",
                                foreground: ShellowTheme.accent,
                                action: onCopyLink
                            )
                        }

                        TerminalIconButton(
                            systemName: "xmark.circle",
                            accessibilityLabel: "Clear Selection",
                            action: clearSelection
                        )
                    }

                    TerminalIconButton(
                        systemName: "doc.on.clipboard",
                        accessibilityLabel: "Paste",
                        action: onPasteClipboard
                    )
                }
                .frame(maxWidth: .infinity, alignment: .leading)
            }
            .scrollIndicators(.hidden)
            .frame(maxWidth: .infinity)

            TerminalIconButton(
                systemName: "return",
                accessibilityLabel: "Enter",
                foreground: .white,
                background: ShellowTheme.accent,
                action: onEnter
            )
            .fixedSize()
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }
}

private struct TerminalIconButton: View {
    let systemName: String
    let accessibilityLabel: String
    var foreground = ShellowTheme.terminalText
    var background = ShellowTheme.keyBackground
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            Image(systemName: systemName)
                .font(.system(size: 15, weight: .semibold))
                .frame(width: 38, height: 38)
        }
        .buttonStyle(.plain)
        .foregroundStyle(foreground)
        .background(background, in: RoundedRectangle(cornerRadius: 8))
        .accessibilityLabel(accessibilityLabel)
    }
}

private struct TerminalJumpToBottomButton: View {
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            Image(systemName: "arrow.down.to.line")
                .font(.system(size: 14, weight: .semibold))
                .frame(width: 38, height: 38)
        }
        .buttonStyle(.plain)
        .foregroundStyle(ShellowTheme.terminalText)
        .background(.thinMaterial, in: RoundedRectangle(cornerRadius: 8))
        .overlay(
            RoundedRectangle(cornerRadius: 8)
                .stroke(ShellowTheme.keyBackground, lineWidth: 1)
        )
        .accessibilityLabel("Jump To Bottom")
    }
}

private struct TerminalKeyboardToolbar: View {
    @Binding var isCtrlArmed: Bool
    @Binding var isAltArmed: Bool
    let applicationCursorKeys: Bool
    let sendInput: (String) -> Void

    var body: some View {
        ScrollView(.horizontal) {
            HStack(spacing: 8) {
                TerminalKeyButton("Esc") { sendInput("\u{1B}") }
                TerminalKeyButton("Tab") { sendInput("\t") }
                TerminalKeyButton("Ctrl", isActive: isCtrlArmed) {
                    isCtrlArmed.toggle()
                    if isCtrlArmed {
                        isAltArmed = false
                    }
                }
                TerminalKeyButton("Alt", isActive: isAltArmed) {
                    isAltArmed.toggle()
                    if isAltArmed {
                        isCtrlArmed = false
                    }
                }
                TerminalKeyButton("^C") { sendWithOptionalAlt("\u{3}") }
                TerminalKeyButton("^D") { sendWithOptionalAlt("\u{4}") }
                TerminalKeyButton("^Z") { sendWithOptionalAlt("\u{1a}") }

                TerminalKeyboardToolbarDivider()

                TerminalKeyButton("^A") { sendWithOptionalAlt("\u{1}") }
                TerminalKeyButton("^B") { sendWithOptionalAlt("\u{2}") }
                TerminalKeyButton("^E") { sendWithOptionalAlt("\u{5}") }
                TerminalKeyButton("^K") { sendWithOptionalAlt("\u{b}") }
                TerminalKeyButton("^O") { sendWithOptionalAlt("\u{f}") }
                TerminalKeyButton("^U") { sendWithOptionalAlt("\u{15}") }
                TerminalKeyButton("^W") { sendWithOptionalAlt("\u{17}") }
                TerminalKeyButton("^R") { sendWithOptionalAlt("\u{12}") }
                TerminalKeyButton("^X") { sendWithOptionalAlt("\u{18}") }
                TerminalKeyButton("^L") { sendWithOptionalAlt("\u{c}") }

                TerminalKeyboardToolbarDivider()

                TerminalIconKey(systemName: "arrow.up") {
                    sendWithOptionalAlt(TerminalArrowKey.up.sequence(applicationCursorKeys: applicationCursorKeys))
                }
                TerminalIconKey(systemName: "arrow.down") {
                    sendWithOptionalAlt(TerminalArrowKey.down.sequence(applicationCursorKeys: applicationCursorKeys))
                }
                TerminalIconKey(systemName: "arrow.left") {
                    sendWithOptionalAlt(TerminalArrowKey.left.sequence(applicationCursorKeys: applicationCursorKeys))
                }
                TerminalIconKey(systemName: "arrow.right") {
                    sendWithOptionalAlt(TerminalArrowKey.right.sequence(applicationCursorKeys: applicationCursorKeys))
                }
                TerminalIconKey(systemName: "delete.left") { sendWithOptionalAlt("\u{7f}") }

                TerminalKeyboardToolbarDivider()

                TerminalKeyButton("Home") { sendWithOptionalAlt("\u{1B}[H") }
                TerminalKeyButton("End") { sendWithOptionalAlt("\u{1B}[F") }
                TerminalKeyButton("PgUp") { sendWithOptionalAlt("\u{1B}[5~") }
                TerminalKeyButton("PgDn") { sendWithOptionalAlt("\u{1B}[6~") }

                TerminalKeyboardToolbarDivider()

                ForEach(TerminalFunctionKey.allCases, id: \.self) { key in
                    TerminalKeyButton(key.title) {
                        sendWithOptionalAlt(key.sequence)
                    }
                }
            }
        }
        .scrollIndicators(.hidden)
        .foregroundStyle(ShellowTheme.terminalText)
    }

    private func sendWithOptionalAlt(_ input: String) {
        if isAltArmed {
            sendInput("\u{1B}" + input)
            isAltArmed = false
        } else {
            sendInput(input)
        }
    }
}

private struct TerminalKeyboardToolbarDivider: View {
    var body: some View {
        Rectangle()
            .fill(ShellowTheme.keyBackground)
            .frame(width: 1, height: 24)
            .padding(.horizontal, 2)
    }
}

private enum TerminalArrowKey {
    case up
    case down
    case left
    case right

    func sequence(applicationCursorKeys: Bool) -> String {
        switch (self, applicationCursorKeys) {
        case (.up, true): "\u{1B}OA"
        case (.down, true): "\u{1B}OB"
        case (.right, true): "\u{1B}OC"
        case (.left, true): "\u{1B}OD"
        case (.up, false): "\u{1B}[A"
        case (.down, false): "\u{1B}[B"
        case (.right, false): "\u{1B}[C"
        case (.left, false): "\u{1B}[D"
        }
    }
}

private enum TerminalFunctionKey: Int, CaseIterable {
    case f1 = 1
    case f2
    case f3
    case f4
    case f5
    case f6
    case f7
    case f8
    case f9
    case f10
    case f11
    case f12

    var title: String {
        "F\(rawValue)"
    }

    var sequence: String {
        switch self {
        case .f1: "\u{1B}OP"
        case .f2: "\u{1B}OQ"
        case .f3: "\u{1B}OR"
        case .f4: "\u{1B}OS"
        case .f5: "\u{1B}[15~"
        case .f6: "\u{1B}[17~"
        case .f7: "\u{1B}[18~"
        case .f8: "\u{1B}[19~"
        case .f9: "\u{1B}[20~"
        case .f10: "\u{1B}[21~"
        case .f11: "\u{1B}[23~"
        case .f12: "\u{1B}[24~"
        }
    }
}

private struct TerminalKeyButton: View {
    let title: String
    var isActive = false
    let action: () -> Void

    init(_ title: String, isActive: Bool = false, action: @escaping () -> Void) {
        self.title = title
        self.isActive = isActive
        self.action = action
    }

    var body: some View {
        Button(action: action) {
            Text(title)
                .font(.caption.weight(.semibold))
                .frame(width: keyWidth, height: 34)
        }
        .buttonStyle(.plain)
        .background(isActive ? ShellowTheme.accent : ShellowTheme.keyBackground, in: RoundedRectangle(cornerRadius: 8))
        .accessibilityLabel(title)
    }

    private var keyWidth: CGFloat {
        title.count > 3 ? 54 : 42
    }
}

private struct TerminalIconKey: View {
    let systemName: String
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            Image(systemName: systemName)
                .font(.system(size: 14, weight: .semibold))
                .frame(width: 44, height: 34)
        }
        .buttonStyle(.plain)
        .background(ShellowTheme.keyBackground, in: RoundedRectangle(cornerRadius: 8))
        .accessibilityLabel(systemName)
    }
}

private struct TerminalDirectInputCapture: UIViewRepresentable {
    let applicationCursorKeys: Bool
    let isEnabled: Bool
    let focusNonce: Int
    let sendInput: (String) -> Void
    let sendTextInput: (String) -> Void
    let sendBackspace: () -> Void
    let handlePaste: (String) -> Void
    let copyShortcut: () -> Void
    let searchShortcut: () -> Void

    func makeUIView(context: Context) -> DirectInputView {
        let view = DirectInputView()
        view.isAccessibilityElement = false
        view.accessibilityElementsHidden = true
        view.applicationCursorKeys = applicationCursorKeys
        view.isInputEnabled = isEnabled
        view.sendInput = sendInput
        view.sendTextInput = sendTextInput
        view.sendBackspace = sendBackspace
        view.handlePaste = handlePaste
        view.copyShortcut = copyShortcut
        view.searchShortcut = searchShortcut
        DispatchQueue.main.async {
            view.updateFirstResponder()
        }
        return view
    }

    func updateUIView(_ uiView: DirectInputView, context: Context) {
        uiView.applicationCursorKeys = applicationCursorKeys
        uiView.isInputEnabled = isEnabled
        uiView.focusNonce = focusNonce
        uiView.sendInput = sendInput
        uiView.sendTextInput = sendTextInput
        uiView.sendBackspace = sendBackspace
        uiView.handlePaste = handlePaste
        uiView.copyShortcut = copyShortcut
        uiView.searchShortcut = searchShortcut
        DispatchQueue.main.async {
            uiView.updateFirstResponder()
        }
    }

    final class DirectInputView: UIView, UIKeyInput {
        var applicationCursorKeys = false
        var isInputEnabled = true
        var focusNonce = 0
        var sendInput: (String) -> Void = { _ in }
        var sendTextInput: (String) -> Void = { _ in }
        var sendBackspace: () -> Void = {}
        var handlePaste: (String) -> Void = { _ in }
        var copyShortcut: () -> Void = {}
        var searchShortcut: () -> Void = {}

        override var canBecomeFirstResponder: Bool { isInputEnabled }
        var hasText: Bool { true }

        var autocapitalizationType: UITextAutocapitalizationType = .none
        var autocorrectionType: UITextAutocorrectionType = .no
        var spellCheckingType: UITextSpellCheckingType = .no
        var smartQuotesType: UITextSmartQuotesType = .no
        var smartDashesType: UITextSmartDashesType = .no
        var smartInsertDeleteType: UITextSmartInsertDeleteType = .no
        var keyboardType: UIKeyboardType = .asciiCapable
        var keyboardAppearance: UIKeyboardAppearance = .dark
        var returnKeyType: UIReturnKeyType = .default

        func updateFirstResponder() {
            if isInputEnabled {
                becomeFirstResponder()
            } else if isFirstResponder {
                resignFirstResponder()
            }
        }

        func insertText(_ text: String) {
            if let sequence = terminalControlSequence(for: text) {
                sendInput(sequence)
                return
            }

            sendTextInput(text)
        }

        func deleteBackward() {
            sendBackspace()
        }

        override func paste(_ sender: Any?) {
            guard let text = UIPasteboard.general.string, !text.isEmpty else { return }
            handlePaste(text)
        }

        override func canPerformAction(_ action: Selector, withSender sender: Any?) -> Bool {
            if action == #selector(paste(_:)) {
                return UIPasteboard.general.hasStrings
            }
            return super.canPerformAction(action, withSender: sender)
        }

        override var keyCommands: [UIKeyCommand]? {
            [
                UIKeyCommand(input: "\r", modifierFlags: [], action: #selector(sendEnter)),
                UIKeyCommand(input: "\t", modifierFlags: [], action: #selector(sendTab)),
                UIKeyCommand(input: "\u{1B}", modifierFlags: [], action: #selector(sendEscape)),
                UIKeyCommand(input: UIKeyCommand.inputUpArrow, modifierFlags: [], action: #selector(sendUp)),
                UIKeyCommand(input: UIKeyCommand.inputDownArrow, modifierFlags: [], action: #selector(sendDown)),
                UIKeyCommand(input: UIKeyCommand.inputLeftArrow, modifierFlags: [], action: #selector(sendLeft)),
                UIKeyCommand(input: UIKeyCommand.inputRightArrow, modifierFlags: [], action: #selector(sendRight)),
                UIKeyCommand(input: UIKeyCommand.inputHome, modifierFlags: [], action: #selector(sendHome)),
                UIKeyCommand(input: UIKeyCommand.inputEnd, modifierFlags: [], action: #selector(sendEnd)),
                UIKeyCommand(input: UIKeyCommand.inputPageUp, modifierFlags: [], action: #selector(sendPageUp)),
                UIKeyCommand(input: UIKeyCommand.inputPageDown, modifierFlags: [], action: #selector(sendPageDown)),
                UIKeyCommand(input: "c", modifierFlags: .control, action: #selector(sendControlC)),
                UIKeyCommand(input: "d", modifierFlags: .control, action: #selector(sendControlD)),
                UIKeyCommand(input: "l", modifierFlags: .control, action: #selector(sendControlL)),
                UIKeyCommand(input: "z", modifierFlags: .control, action: #selector(sendControlZ)),
                UIKeyCommand(input: "[", modifierFlags: .control, action: #selector(sendEscape)),
                UIKeyCommand(input: "c", modifierFlags: .command, action: #selector(copyCommand)),
                UIKeyCommand(input: "v", modifierFlags: .command, action: #selector(pasteCommand)),
                UIKeyCommand(input: "f", modifierFlags: .command, action: #selector(searchCommand))
            ]
        }

        override func pressesBegan(_ presses: Set<UIPress>, with event: UIPressesEvent?) {
            var handledAll = true

            for press in presses {
                guard let key = press.key else {
                    handledAll = false
                    continue
                }

                let charactersIgnoringModifiers = key.charactersIgnoringModifiers
                let characters = charactersIgnoringModifiers.isEmpty ? key.characters : charactersIgnoringModifiers

                if let sequence = terminalControlSequence(for: characters) {
                    if key.modifierFlags.contains(.alternate) {
                        sendInput("\u{1B}" + sequence)
                    } else {
                        sendInput(sequence)
                    }
                } else if key.modifierFlags.contains(.control),
                   let encoded = controlEncoded(from: key.charactersIgnoringModifiers) {
                    sendInput(encoded)
                } else if key.modifierFlags.contains(.alternate),
                          let encoded = metaEncoded(from: key.charactersIgnoringModifiers) {
                    sendInput(encoded)
                } else if key.modifierFlags.contains(.command) {
                    handledAll = false
                } else if !key.characters.isEmpty {
                    sendInput(key.characters)
                } else {
                    handledAll = false
                }
            }

            if !handledAll {
                super.pressesBegan(presses, with: event)
            }
        }

        @objc private func sendEnter() { sendInput("\r") }
        @objc private func sendTab() { sendInput("\t") }
        @objc private func sendEscape() { sendInput("\u{1B}") }
        @objc private func sendUp() { sendInput(TerminalArrowKey.up.sequence(applicationCursorKeys: applicationCursorKeys)) }
        @objc private func sendDown() { sendInput(TerminalArrowKey.down.sequence(applicationCursorKeys: applicationCursorKeys)) }
        @objc private func sendLeft() { sendInput(TerminalArrowKey.left.sequence(applicationCursorKeys: applicationCursorKeys)) }
        @objc private func sendRight() { sendInput(TerminalArrowKey.right.sequence(applicationCursorKeys: applicationCursorKeys)) }
        @objc private func sendHome() { sendInput("\u{1B}[H") }
        @objc private func sendEnd() { sendInput("\u{1B}[F") }
        @objc private func sendPageUp() { sendInput("\u{1B}[5~") }
        @objc private func sendPageDown() { sendInput("\u{1B}[6~") }
        @objc private func sendControlC() { sendInput("\u{3}") }
        @objc private func sendControlD() { sendInput("\u{4}") }
        @objc private func sendControlL() { sendInput("\u{c}") }
        @objc private func sendControlZ() { sendInput("\u{1a}") }
        @objc private func copyCommand() { copyShortcut() }
        @objc private func pasteCommand() {
            guard let text = UIPasteboard.general.string, !text.isEmpty else { return }
            handlePaste(text)
        }
        @objc private func searchCommand() { searchShortcut() }

        private func terminalControlSequence(for input: String) -> String? {
            switch input {
            case UIKeyCommand.inputUpArrow:
                TerminalArrowKey.up.sequence(applicationCursorKeys: applicationCursorKeys)
            case UIKeyCommand.inputDownArrow:
                TerminalArrowKey.down.sequence(applicationCursorKeys: applicationCursorKeys)
            case UIKeyCommand.inputLeftArrow:
                TerminalArrowKey.left.sequence(applicationCursorKeys: applicationCursorKeys)
            case UIKeyCommand.inputRightArrow:
                TerminalArrowKey.right.sequence(applicationCursorKeys: applicationCursorKeys)
            case UIKeyCommand.inputHome:
                "\u{1B}[H"
            case UIKeyCommand.inputEnd:
                "\u{1B}[F"
            case UIKeyCommand.inputPageUp:
                "\u{1B}[5~"
            case UIKeyCommand.inputPageDown:
                "\u{1B}[6~"
            default:
                nil
            }
        }

        private func controlEncoded(from characters: String) -> String? {
            guard let scalar = characters.lowercased().unicodeScalars.first,
                  CharacterSet.lowercaseLetters.contains(scalar),
                  let control = UnicodeScalar(scalar.value - 96)
            else {
                return nil
            }
            return String(control)
        }

        private func metaEncoded(from characters: String) -> String? {
            guard !characters.isEmpty else { return nil }
            return characters.map { "\u{1B}\($0)" }.joined()
        }
    }
}

private extension String {
    var lineCount: Int {
        guard !isEmpty else { return 0 }
        return reduce(1) { count, character in
            count + (character.isNewline ? 1 : 0)
        }
    }

    var isRiskyTerminalPaste: Bool {
        count > 120 || contains(where: \.isNewline)
    }

    var safeTranscriptFileComponent: String {
        let allowed = CharacterSet.alphanumerics.union(CharacterSet(charactersIn: "-_"))
        let transformed = unicodeScalars.map { scalar in
            allowed.contains(scalar) ? Character(scalar) : "-"
        }
        let value = String(transformed)
            .split(separator: "-")
            .joined(separator: "-")
            .trimmingCharacters(in: CharacterSet(charactersIn: "-"))
        return value.isEmpty ? "terminal" : value
    }
}

private extension TerminalSelection {
    var rows: ClosedRange<Int> {
        switch self {
        case .grid(let anchor, let focus):
            return min(anchor.row, focus.row)...max(anchor.row, focus.row)
        case .history(let anchor, let focus):
            return min(anchor, focus)...max(anchor, focus)
        }
    }

    func containsGrid(row: Int) -> Bool {
        guard case .grid = self else { return false }
        return rows.contains(row)
    }

    func containsHistory(row: Int) -> Bool {
        guard case .history = self else { return false }
        return rows.contains(row)
    }

    static func gridRow(_ row: Int) -> TerminalSelection {
        .grid(
            anchor: TerminalSelectionPoint(row: row, column: 0),
            focus: TerminalSelectionPoint(row: row, column: TerminalSelectionPoint.lineEndColumn)
        )
    }

    func extendingGridRow(to row: Int) -> TerminalSelection {
        if case .grid(let anchor, _) = self {
            return .grid(
                anchor: TerminalSelectionPoint(row: anchor.row, column: 0),
                focus: TerminalSelectionPoint(row: row, column: TerminalSelectionPoint.lineEndColumn)
            )
        }
        return .gridRow(row)
    }

    func extendingHistory(to row: Int) -> TerminalSelection {
        if case .history(let anchor, _) = self {
            return .history(anchor: anchor, focus: row)
        }
        return .history(anchor: row, focus: row)
    }

    func gridCellRange(row: Int, line: String) -> Range<Int>? {
        guard case .grid(let anchor, let focus) = self else { return nil }
        let lineEnd = max(line.terminalCellWidth, 1)
        let (start, end) = ordered(anchor, focus)
        guard row >= start.row, row <= end.row else { return nil }

        let lower: Int
        if row == start.row {
            lower = min(start.column, lineEnd)
        } else {
            lower = 0
        }

        let upper: Int
        if row == end.row {
            upper = min(end.column, lineEnd)
        } else {
            upper = lineEnd
        }

        if lower == upper, row == start.row, row == end.row {
            return nil
        }
        return min(lower, upper)..<max(lower, upper)
    }

    func isFullGridRow(row: Int, line: String) -> Bool {
        guard let range = gridCellRange(row: row, line: line) else { return false }
        let lineEnd = max(line.terminalCellWidth, 1)
        return range.lowerBound <= 0 && range.upperBound >= lineEnd
    }

    private func ordered(
        _ lhs: TerminalSelectionPoint,
        _ rhs: TerminalSelectionPoint
    ) -> (TerminalSelectionPoint, TerminalSelectionPoint) {
        if lhs.row < rhs.row || (lhs.row == rhs.row && lhs.column <= rhs.column) {
            return (lhs, rhs)
        }
        return (rhs, lhs)
    }
}

private extension TerminalSession {
    func cursorBottomY(
        fontSize: Double,
        lineHeightScale: Double,
        viewportHeight: CGFloat,
        topOffset: CGFloat
    ) -> CGFloat? {
        let rowHeight = TerminalMetrics.rowHeight(fontSize: fontSize, lineHeightScale: lineHeightScale)
        let contentHeight = max(rowHeight, viewportHeight - TerminalMetrics.viewportVerticalPadding * 2)
        let visibleRowCapacity = max(1, Int(contentHeight / rowHeight))

        guard let row = focusedCursorViewportRow(visibleRowCapacity: visibleRowCapacity) else {
            return nil
        }

        return topOffset
            + TerminalMetrics.viewportVerticalPadding
            + CGFloat(row + 1) * rowHeight
    }

    private func focusedCursorViewportRow(visibleRowCapacity: Int) -> Int? {
        if let grid, grid.hasVisibleContent || grid.activeScreen == .alternate {
            guard grid.cursorVisible else { return nil }

            let totalRows = max(grid.lines.count, Int(grid.rows))
            let cursorRow = max(0, min(grid.cursorRow, max(0, totalRows - 1)))
            let firstVisibleRow: Int

            if grid.activeScreen == .alternate {
                firstVisibleRow = 0
            } else {
                let maxFirstRow = max(0, totalRows - visibleRowCapacity)
                firstVisibleRow = max(0, min(cursorRow - visibleRowCapacity + 1, maxFirstRow))
            }

            return max(0, min(cursorRow - firstVisibleRow, visibleRowCapacity - 1))
        }

        guard !rows.isEmpty else { return nil }

        let totalRows = max(rows.count, terminalRows)
        let cursorRow = rows.count - 1
        let maxFirstRow = max(0, totalRows - visibleRowCapacity)
        let firstVisibleRow = max(0, min(cursorRow - visibleRowCapacity + 1, maxFirstRow))
        return max(0, min(cursorRow - firstVisibleRow, visibleRowCapacity - 1))
    }

    var isAlternateScreenActive: Bool {
        guard let grid else { return false }
        return grid.activeScreen == .alternate
    }

    var isBracketedPasteActive: Bool {
        grid?.bracketedPaste == true
    }

    var isApplicationCursorKeysActive: Bool {
        grid?.applicationCursorKeys == true
    }

    var promptInputText: String {
        guard !isAlternateScreenActive, let row = rows.last, row.style == .prompt else { return "" }
        return row.text
    }

    var copyableText: String {
        if let grid, grid.hasVisibleContent || grid.activeScreen == .alternate {
            return grid.lines.joined(separator: "\n")
        }

        return rows.map { row in
            let prompt = row.prompt.isEmpty ? "" : "\(row.prompt) "
            return prompt + row.text
        }
        .joined(separator: "\n")
    }

    func selectedText(for selection: TerminalSelection?) -> String? {
        guard let selection else { return nil }

        switch selection {
        case .grid:
            guard let grid else { return nil }
            let text = selection.rows
                .compactMap { row -> String? in
                    guard grid.lines.indices.contains(row) else { return nil }
                    let line = grid.lines[row]
                    guard let range = selection.gridCellRange(row: row, line: line) else { return nil }
                    return line.terminalSubstring(cells: range).trimmingCharacters(in: .whitespaces)
                }
                .joined(separator: "\n")
                .trimmingCharacters(in: .whitespacesAndNewlines)
            return text.isEmpty ? nil : text
        case .history:
            let text = selection.rows
                .compactMap { row -> String? in
                    guard rows.indices.contains(row) else { return nil }
                    let terminalRow = rows[row]
                    let prompt = terminalRow.prompt.isEmpty ? "" : "\(terminalRow.prompt) "
                    return (prompt + terminalRow.text).trimmingCharacters(in: .whitespaces)
                }
                .joined(separator: "\n")
                .trimmingCharacters(in: .whitespacesAndNewlines)
            return text.isEmpty ? nil : text
        }
    }

    func searchPresentation(query: String, focusedIndex: Int) -> TerminalSearchPresentation {
        let normalized = query.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !normalized.isEmpty else {
            return TerminalSearchPresentation(query: "", hits: [], activeHit: nil)
        }

        let hits: [TerminalSearchHit]
        if let grid, grid.hasVisibleContent || grid.activeScreen == .alternate {
            hits = grid.lines.enumerated().flatMap { index, line in
                line.terminalSearchCellRanges(query: normalized).map { range in
                    TerminalSearchHit.grid(row: index, start: range.lowerBound, end: range.upperBound)
                }
            }
        } else {
            hits = rows.enumerated().compactMap { index, row in
                row.searchableText.localizedCaseInsensitiveContains(normalized) ? .history(index) : nil
            }
        }

        let activeHit = hits.isEmpty ? nil : hits[focusedIndex.modulo(hits.count)]
        return TerminalSearchPresentation(query: normalized, hits: hits, activeHit: activeHit)
    }
}

private extension TerminalRow {
    var searchableText: String {
        let prompt = prompt.isEmpty ? "" : "\(prompt) "
        return prompt + text
    }
}

private extension Int {
    func modulo(_ divisor: Int) -> Int {
        guard divisor > 0 else { return 0 }
        let remainder = self % divisor
        return remainder >= 0 ? remainder : remainder + divisor
    }
}

private enum TerminalMouseEvent: Equatable {
    case press
    case drag
    case release

    var buttonCode: Int {
        switch self {
        case .press, .release: 0
        case .drag: 32
        }
    }

    var terminator: Character {
        switch self {
        case .release: "m"
        case .press, .drag: "M"
        }
    }
}

private extension TerminalGridSnapshot {
    func mousePressSequence(row: Int, column: Int) -> String? {
        mouseEventSequence(row: row, column: column, event: .press)
    }

    func mouseEventSequence(row: Int, column: Int, event: TerminalMouseEvent) -> String? {
        guard mouseReporting, sgrMouse else { return nil }
        if event == .drag, !mouseDragReporting {
            return nil
        }

        let terminalRow: Int
        if activeScreen == .primary {
            terminalRow = row - scrollbackLen + 1
        } else {
            terminalRow = row + 1
        }

        guard terminalRow >= 1, terminalRow <= rows else { return nil }
        let terminalColumn = max(1, min(column + 1, cols))
        return "\u{1B}[<\(event.buttonCode);\(terminalColumn);\(terminalRow)\(event.terminator)"
    }
}

private extension TerminalGridStyle {
    static let plain = TerminalGridStyle(
        bold: false,
        faint: false,
        italic: false,
        underline: false,
        blink: false,
        inverse: false,
        strikethrough: false,
        fg: nil,
        bg: nil
    )
}

private extension TerminalGridColor {
    var color: Color {
        Color(
            red: Double(r) / 255.0,
            green: Double(g) / 255.0,
            blue: Double(b) / 255.0
        )
    }
}

enum ShellowTheme {
    static let accent = Color(red: 0.11, green: 0.62, blue: 0.44)
    static let success = Color(red: 0.27, green: 0.82, blue: 0.55)
    static let warning = Color(red: 0.93, green: 0.68, blue: 0.22)
    static let muted = Color(red: 0.54, green: 0.58, blue: 0.64)
    static let prompt = Color(red: 0.46, green: 0.86, blue: 0.67)
    static let terminalText = Color(red: 0.88, green: 0.91, blue: 0.86)
    static let terminalMuted = Color(red: 0.58, green: 0.64, blue: 0.61)
    static let terminalBackground = Color(red: 0.05, green: 0.06, blue: 0.06)
    static let panelBackground = Color(red: 0.08, green: 0.09, blue: 0.09)
    static let inputBackground = Color(red: 0.12, green: 0.13, blue: 0.13)
    static let keyBackground = Color(red: 0.15, green: 0.16, blue: 0.16)
    static let selectionBackground = Color(red: 0.18, green: 0.45, blue: 0.38).opacity(0.72)
    static let searchBackground = Color(red: 0.52, green: 0.44, blue: 0.15).opacity(0.44)
    static let searchCurrentBackground = Color(red: 0.79, green: 0.62, blue: 0.18).opacity(0.72)
}

#Preview {
    TerminalScreen(
        session: .constant(.preview),
        settings: ShellowSettings(),
        renderTick: 0,
        onTerminalInput: { _ in },
        onReconnect: nil,
        onDisconnect: {},
        onResizeTerminal: { _, _ in },
        onAttachRendererSurface: { _, _, _ in },
        onSetRendererOverlay: { _ in },
        onRenderRendererSurface: { _, _, _, _ in false },
        onDetachRendererSurface: {},
        onClearTerminal: {},
        onResetTerminal: {}
    )
}
