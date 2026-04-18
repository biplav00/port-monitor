import SwiftUI
import ServiceManagement

struct SettingsView: View {
    @EnvironmentObject var scanner: PortScanner
    @EnvironmentObject var settings: AppSettings
    let onDone: () -> Void

    @State private var portMinText = ""
    @State private var portMaxText = ""
    @State private var rangeError: String? = nil

    private let intervals: [Double] = [1, 3, 5, 10]

    var body: some View {
        VStack(spacing: 0) {
            settingsHeader
            Divider().opacity(0.08)
            settingsBody
        }
        .onAppear {
            portMinText = "\(settings.portRangeMin)"
            portMaxText = "\(settings.portRangeMax)"
        }
    }

    private var settingsHeader: some View {
        HStack {
            Text("Settings")
                .font(.system(size: 13, weight: .semibold))
                .foregroundColor(.primary.opacity(0.85))
            Spacer()
            Button("Done", action: onDone)
                .font(.system(size: 13, weight: .medium))
                .foregroundColor(.blue.opacity(0.8))
                .buttonStyle(.plain)
        }
        .padding(.horizontal, 13)
        .padding(.top, 10)
        .padding(.bottom, 9)
    }

    private var settingsBody: some View {
        VStack(alignment: .leading, spacing: 0) {
            sectionLabel("REFRESH INTERVAL")
            intervalPicker
            divider()
            sectionLabel("PORT RANGE")
            portRangePicker
            if let err = rangeError {
                Text(err)
                    .font(.system(size: 10))
                    .foregroundColor(.red.opacity(0.7))
                    .padding(.horizontal, 13)
                    .padding(.bottom, 4)
            }
            divider()
            sectionLabel("APPEARANCE")
            appearancePicker
            divider()
            launchAtLoginRow
            divider()
            quitButton
        }
    }

    private var appearancePicker: some View {
        HStack(spacing: 2) {
            ForEach(AppearanceMode.allCases, id: \.self) { mode in
                Button(mode.label) {
                    settings.appearance = mode
                }
                .font(.system(size: 12))
                .foregroundColor(settings.appearance == mode
                    ? .primary.opacity(0.85)
                    : .primary.opacity(0.4))
                .padding(.horizontal, 10)
                .padding(.vertical, 3)
                .background(settings.appearance == mode
                    ? Color.primary.opacity(0.14)
                    : Color.clear)
                .cornerRadius(5)
                .buttonStyle(.plain)
            }
        }
        .padding(.horizontal, 13)
        .padding(.bottom, 8)
    }

    private var intervalPicker: some View {
        HStack(spacing: 2) {
            ForEach(intervals, id: \.self) { interval in
                Button("\(Int(interval))s") {
                    settings.refreshInterval = interval
                    scanner.scheduleTimer()
                }
                .font(.system(size: 12))
                .foregroundColor(settings.refreshInterval == interval
                    ? .primary.opacity(0.85)
                    : .primary.opacity(0.4))
                .padding(.horizontal, 10)
                .padding(.vertical, 3)
                .background(settings.refreshInterval == interval
                    ? Color.primary.opacity(0.14)
                    : Color.clear)
                .cornerRadius(5)
                .buttonStyle(.plain)
            }
        }
        .padding(.horizontal, 13)
        .padding(.bottom, 8)
    }

    private var portRangePicker: some View {
        HStack(spacing: 6) {
            TextField("", text: $portMinText)
                .textFieldStyle(.roundedBorder)
                .frame(width: 54)
                .font(.system(size: 12, design: .monospaced))
                .onSubmit { applyRange() }

            Text("–").foregroundColor(.primary.opacity(0.25))

            TextField("", text: $portMaxText)
                .textFieldStyle(.roundedBorder)
                .frame(width: 54)
                .font(.system(size: 12, design: .monospaced))
                .onSubmit { applyRange() }

            Button("Apply") { applyRange() }
                .font(.system(size: 11))
                .foregroundColor(.blue.opacity(0.8))
                .buttonStyle(.plain)
        }
        .padding(.horizontal, 13)
        .padding(.bottom, 8)
    }

    private var launchAtLoginRow: some View {
        HStack {
            Text("Launch at Login")
                .font(.system(size: 13))
                .foregroundColor(.primary.opacity(0.75))
            Spacer()
            Toggle("", isOn: Binding(
                get: { SMAppService.mainApp.status == .enabled },
                set: { shouldEnable in
                    do {
                        if shouldEnable {
                            try SMAppService.mainApp.register()
                        } else {
                            try SMAppService.mainApp.unregister()
                        }
                    } catch {
                        print("SMAppService error: \(error)")
                    }
                }
            ))
            .labelsHidden()
            .toggleStyle(.switch)
        }
        .padding(.horizontal, 13)
        .padding(.vertical, 8)
    }

    private var quitButton: some View {
        Button("Quit Port Monitor") {
            NSApplication.shared.terminate(nil)
        }
        .font(.system(size: 13))
        .foregroundColor(.red.opacity(0.7))
        .buttonStyle(.plain)
        .padding(.horizontal, 13)
        .padding(.vertical, 8)
    }

    private func sectionLabel(_ text: String) -> some View {
        Text(text)
            .font(.system(size: 10, weight: .medium))
            .foregroundColor(.primary.opacity(0.25))
            .padding(.horizontal, 13)
            .padding(.top, 10)
            .padding(.bottom, 6)
    }

    private func divider() -> some View {
        Divider().opacity(0.05).padding(.vertical, 2)
    }

    private func applyRange() {
        guard let min = Int(portMinText), let max = Int(portMaxText) else {
            rangeError = "Enter valid port numbers"
            return
        }
        guard min >= 0, max <= 65535, min < max else {
            rangeError = "Must be 0–65535 with min < max"
            return
        }
        rangeError = nil
        settings.portRangeMin = min
        settings.portRangeMax = max
        scanner.refresh()
    }
}
