import SwiftUI

struct SettingsScreen: View {
    @Binding var settings: ShellowSettings

    var body: some View {
        NavigationStack {
            Form {
                Section("Terminal") {
                    Picker("Theme", selection: $settings.colorScheme) {
                        ForEach(TerminalColorScheme.allCases) { scheme in
                            Text(scheme.title).tag(scheme)
                        }
                    }

                    VStack(alignment: .leading, spacing: 10) {
                        HStack {
                            Text("Font Size")
                            Spacer()
                            Text(settings.fontSize, format: .number.precision(.fractionLength(0)))
                                .foregroundStyle(.secondary)
                        }
                        Slider(value: $settings.fontSize, in: ShellowSettings.fontSizeRange, step: 1)
                    }

                    VStack(alignment: .leading, spacing: 10) {
                        HStack {
                            Text("Line Height")
                            Spacer()
                            Text("\(Int(settings.lineHeightScale * 100))%")
                                .foregroundStyle(.secondary)
                        }
                        Slider(value: $settings.lineHeightScale, in: ShellowSettings.lineHeightScaleRange, step: 0.05)
                    }
                }

                Section("Input") {
                    Toggle("Keyboard Toolbar", isOn: $settings.showKeyboardToolbar)
                    Toggle("Confirm Paste", isOn: $settings.confirmPaste)
                }

                Section("Transport") {
                    VStack(alignment: .leading, spacing: 10) {
                        HStack {
                            Text("Keep Alive")
                            Spacer()
                            Text("\(Int(settings.keepAliveSeconds))s")
                                .foregroundStyle(.secondary)
                        }
                        Slider(value: $settings.keepAliveSeconds, in: ShellowSettings.keepAliveRange, step: 5)
                    }
                }
            }
            .navigationTitle("Settings")
        }
    }
}

#Preview {
    SettingsScreen(settings: .constant(ShellowSettings()))
}
