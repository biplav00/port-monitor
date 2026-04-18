import SwiftUI

struct PopoverView: View {
    @EnvironmentObject var scanner: PortScanner
    @EnvironmentObject var settings: AppSettings
    @State private var showSettings = false

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider().opacity(0.08)

            if showSettings {
                SettingsView(onDone: { showSettings = false })
            } else {
                portListSection
            }
        }
        .frame(width: 300)
        .preferredColorScheme(.dark)
    }

    private var header: some View {
        HStack {
            Text("Port Monitor")
                .font(.system(size: 13, weight: .semibold))
                .foregroundColor(.primary.opacity(0.85))
            Spacer()
            Text("⚙")
                .font(.system(size: 18))
                .foregroundColor(.primary.opacity(0.3))
                .frame(width: 24, height: 24)
                .contentShape(Rectangle())
                .onTapGesture { showSettings.toggle() }
        }
        .padding(.horizontal, 13)
        .padding(.top, 10)
        .padding(.bottom, 9)
    }

    @ViewBuilder
    private var portListSection: some View {
        if scanner.ports.isEmpty {
            emptyState
        } else {
            columnHeaders
            Divider().opacity(0.05)
            portRows
            Divider().opacity(0.07)
            footer
        }
    }

    private var emptyState: some View {
        Text("No ports listening")
            .font(.system(size: 12))
            .foregroundColor(.secondary)
            .padding(.vertical, 20)
            .frame(maxWidth: .infinity)
    }

    private var columnHeaders: some View {
        HStack(spacing: 0) {
            Color.clear.frame(width: 14)
            Text("Port")
                .frame(width: 56, alignment: .leading)
            Text("Process")
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.leading, 10)
            Text("Action")
                .frame(width: 44, alignment: .trailing)
        }
        .font(.system(size: 10))
        .foregroundColor(.primary.opacity(0.2))
        .padding(.horizontal, 13)
        .padding(.vertical, 5)
    }

    private var portRows: some View {
        ForEach(scanner.ports) { entry in
            PortRowView(entry: entry) {
                scanner.kill(entry: entry)
            }
            if entry.id != scanner.ports.last?.id {
                Divider()
                    .opacity(0.05)
                    .padding(.horizontal, 13)
            }
        }
    }

    private var footer: some View {
        HStack {
            Text("\(scanner.ports.count) port\(scanner.ports.count == 1 ? "" : "s") · every \(intervalLabel)")
                .font(.system(size: 11))
                .foregroundColor(.primary.opacity(0.3))
            Spacer()
        }
        .padding(.horizontal, 13)
        .padding(.vertical, 8)
    }

    private var intervalLabel: String {
        let i = settings.refreshInterval
        return i == Double(Int(i)) ? "\(Int(i))s" : "\(i)s"
    }
}
