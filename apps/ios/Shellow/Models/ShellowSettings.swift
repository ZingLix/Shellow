import Foundation
import SwiftUI

struct ShellowSettings: Equatable, Codable {
    static let fontSizeRange: ClosedRange<Double> = 11...18
    static let lineHeightScaleRange: ClosedRange<Double> = 0.9...1.25
    static let keepAliveRange: ClosedRange<Double> = 10...120

    var fontSize: Double = 13
    var lineHeightScale: Double = 1.0
    var confirmPaste = true
    var showKeyboardToolbar = true
    var keepAliveSeconds: Double = 30
    var colorScheme: TerminalColorScheme = .system

    func normalized() -> ShellowSettings {
        var copy = self
        copy.fontSize = copy.fontSize.clamped(to: Self.fontSizeRange)
        copy.lineHeightScale = copy.lineHeightScale.clamped(to: Self.lineHeightScaleRange)
        copy.keepAliveSeconds = copy.keepAliveSeconds.clamped(to: Self.keepAliveRange)
        return copy
    }
}

private extension Comparable {
    func clamped(to range: ClosedRange<Self>) -> Self {
        min(max(self, range.lowerBound), range.upperBound)
    }
}

enum TerminalColorScheme: String, CaseIterable, Identifiable, Codable {
    case dark
    case light
    case system

    var id: String { rawValue }

    var title: String {
        switch self {
        case .dark: "Dark"
        case .light: "Light"
        case .system: "System"
        }
    }
}

extension TerminalColorScheme {
    var preferredSwiftUIColorScheme: ColorScheme? {
        switch self {
        case .dark: .dark
        case .light: .light
        case .system: nil
        }
    }
}

enum ShellowSettingsStore {
    private static let key = "shellow.settings.v1"

    static func load() -> ShellowSettings {
        guard let data = UserDefaults.standard.data(forKey: key) else {
            return ShellowSettings()
        }

        do {
            return try JSONDecoder().decode(ShellowSettings.self, from: data).normalized()
        } catch {
            return ShellowSettings()
        }
    }

    static func save(_ settings: ShellowSettings) {
        guard let data = try? JSONEncoder().encode(settings.normalized()) else {
            return
        }
        UserDefaults.standard.set(data, forKey: key)
    }
}
