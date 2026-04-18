import SwiftUI

struct PortRowView: View {
    let entry: PortEntry
    let onKill: () -> Bool

    @State private var isHovered = false
    @State private var showKillError = false

    var body: some View {
        HStack(spacing: 0) {
            Circle()
                .fill(entry.isSafeToKill
                    ? Color(red: 0.19, green: 0.82, blue: 0.35)
                    : Color(red: 0.96, green: 0.62, blue: 0.15))
                .frame(width: 6, height: 6)
                .frame(width: 14)
                .help(entry.isSafeToKill
                    ? "Your process — safe to kill"
                    : "Owned by \(entry.user) — may require elevated privileges to kill")

            Text("\(entry.port)")
                .font(.system(size: 13, weight: .medium))
                .foregroundColor(.primary.opacity(0.85))
                .monospacedDigit()
                .frame(width: 56, alignment: .leading)

            Text("\(entry.processName) · \(entry.pid)")
                .font(.system(size: 12))
                .foregroundColor(.primary.opacity(0.3))
                .lineLimit(1)
                .truncationMode(.tail)
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.leading, 10)

            Button("Kill") {
                if !onKill() { showKillError = true }
            }
            .font(.system(size: 11, weight: .medium))
            .foregroundColor(isHovered ? Color(red: 1, green: 0.27, blue: 0.23) : .clear)
            .frame(width: 44, alignment: .trailing)
            .buttonStyle(.plain)
        }
        .padding(.horizontal, 13)
        .padding(.vertical, 6)
        .background(isHovered ? Color.primary.opacity(0.05) : Color.clear)
        .onHover { isHovered = $0 }
        .alert("Could not kill process", isPresented: $showKillError) {
            Button("OK", role: .cancel) {}
        } message: {
            Text("Permission denied for PID \(entry.pid). Try running Port Monitor with elevated privileges.")
        }
    }
}
