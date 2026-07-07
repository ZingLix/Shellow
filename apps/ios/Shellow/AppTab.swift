import SwiftUI

enum AppTab: String, CaseIterable, Identifiable {
    case terminal
    case hosts
    case settings

    var id: String { rawValue }

    var title: String {
        switch self {
        case .terminal: "Terminal"
        case .hosts: "Hosts"
        case .settings: "Settings"
        }
    }

    var systemImage: String {
        switch self {
        case .terminal: "terminal"
        case .hosts: "server.rack"
        case .settings: "gearshape"
        }
    }

    @ViewBuilder
    var label: some View {
        Label(title, systemImage: systemImage)
    }
}
