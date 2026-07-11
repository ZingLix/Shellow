import SwiftUI

struct SettingsScreen: View {
    @Binding var settings: ShellowSettings
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            Form {
                Section("Terminal") {
                    Picker("Terminal Theme", selection: $settings.terminalTheme) {
                        ForEach(TerminalThemeSelection.allCases) { theme in
                            Text(theme.title).tag(theme)
                        }
                    }

                    Picker("App Appearance", selection: $settings.colorScheme) {
                        ForEach(TerminalColorScheme.allCases) { scheme in
                            Text(scheme.title).tag(scheme)
                        }
                    }

                    SettingsSliderRow(
                        title: "Font Size",
                        valueText: settings.fontSize.formatted(.number.precision(.fractionLength(0))),
                        value: $settings.fontSize,
                        range: ShellowSettings.fontSizeRange,
                        step: 1
                    )

                    SettingsSliderRow(
                        title: "Line Height",
                        valueText: "\(Int(settings.lineHeightScale * 100))%",
                        value: $settings.lineHeightScale,
                        range: ShellowSettings.lineHeightScaleRange,
                        step: 0.05
                    )
                }

                Section("Input") {
                    Toggle("Keyboard Toolbar", isOn: $settings.showKeyboardToolbar)
                    Toggle("Confirm Paste", isOn: $settings.confirmPaste)
                }

                Section("Transport") {
                    SettingsSliderRow(
                        title: "Keep Alive",
                        valueText: "\(Int(settings.keepAliveSeconds))s",
                        value: $settings.keepAliveSeconds,
                        range: ShellowSettings.keepAliveRange,
                        step: 5
                    )
                    Toggle("Detect Remote Ports", isOn: $settings.detectRemotePorts)
                    Text("Optional. Opens a second SSH channel and checks listening TCP ports every two seconds. No ports are forwarded automatically.")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                }

                Section {
                    Button("Restore Defaults", role: .destructive) {
                        settings = ShellowSettings()
                    }
                }
            }
            .navigationTitle("Settings")
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button("Done") {
                        dismiss()
                    }
                }
            }
        }
    }
}

private struct SettingsSliderRow: View {
    let title: String
    let valueText: String
    @Binding var value: Double
    let range: ClosedRange<Double>
    let step: Double

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            LabeledContent(title) {
                Text(valueText)
                    .font(.subheadline.monospacedDigit())
                    .foregroundStyle(.secondary)
            }

            Slider(value: $value, in: range, step: step)
                .tint(ShellowTheme.accent)
                .accessibilityLabel(title)
                .accessibilityValue(valueText)
        }
        .padding(.vertical, 2)
    }
}

#Preview {
    SettingsScreen(settings: .constant(ShellowSettings()))
}
