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
    var terminalTheme: TerminalThemeSelection = .shellowDark

    private enum CodingKeys: String, CodingKey {
        case fontSize
        case lineHeightScale
        case confirmPaste
        case showKeyboardToolbar
        case keepAliveSeconds
        case colorScheme
        case terminalTheme
    }

    init() {}

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        fontSize = try container.decodeIfPresent(Double.self, forKey: .fontSize) ?? 13
        lineHeightScale = try container.decodeIfPresent(Double.self, forKey: .lineHeightScale) ?? 1.0
        confirmPaste = try container.decodeIfPresent(Bool.self, forKey: .confirmPaste) ?? true
        showKeyboardToolbar = try container.decodeIfPresent(Bool.self, forKey: .showKeyboardToolbar) ?? true
        keepAliveSeconds = try container.decodeIfPresent(Double.self, forKey: .keepAliveSeconds) ?? 30
        colorScheme = try container.decodeIfPresent(TerminalColorScheme.self, forKey: .colorScheme) ?? .system
        terminalTheme = try container.decodeIfPresent(TerminalThemeSelection.self, forKey: .terminalTheme) ?? .shellowDark
    }

    func normalized() -> ShellowSettings {
        var copy = self
        copy.fontSize = copy.fontSize.clamped(to: Self.fontSizeRange)
        copy.lineHeightScale = copy.lineHeightScale.clamped(to: Self.lineHeightScaleRange)
        copy.keepAliveSeconds = copy.keepAliveSeconds.clamped(to: Self.keepAliveRange)
        return copy
    }
}

enum TerminalThemeSelection: String, CaseIterable, Identifiable, Codable {
    case shellowDark = "shellow_dark"
    case midnight
    case amber
    case paperLight = "paper_light"

    var id: String { rawValue }

    var title: String {
        switch self {
        case .shellowDark: "Shellow Dark"
        case .midnight: "Midnight"
        case .amber: "Amber"
        case .paperLight: "Paper Light"
        }
    }

    var backgroundColor: Color {
        switch self {
        case .shellowDark: Color(red: 0.051, green: 0.059, blue: 0.055)
        case .midnight: Color(red: 0.043, green: 0.071, blue: 0.125)
        case .amber: Color(red: 0.090, green: 0.075, blue: 0.051)
        case .paperLight: Color(red: 0.980, green: 0.973, blue: 0.949)
        }
    }

    var metalBackground: (red: Double, green: Double, blue: Double) {
        switch self {
        case .shellowDark: (0x0D / 255.0, 0x0F / 255.0, 0x0E / 255.0)
        case .midnight: (0x0B / 255.0, 0x12 / 255.0, 0x20 / 255.0)
        case .amber: (0x17 / 255.0, 0x13 / 255.0, 0x0D / 255.0)
        case .paperLight: (0xFA / 255.0, 0xF8 / 255.0, 0xF2 / 255.0)
        }
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
