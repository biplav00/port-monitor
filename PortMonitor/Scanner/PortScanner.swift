import Foundation
import Combine

final class PortScanner: ObservableObject {
    @Published var ports: [PortEntry] = []

    private let settings: AppSettings
    private var timer: AnyCancellable?

    init(settings: AppSettings = .shared) {
        self.settings = settings
    }

    func start() {
        refresh()
        scheduleTimer()
    }

    func stop() {
        timer?.cancel()
        timer = nil
    }

    func scheduleTimer() {
        timer?.cancel()
        timer = Timer.publish(every: settings.refreshInterval, on: .main, in: .common)
            .autoconnect()
            .sink { [weak self] _ in self?.refresh() }
    }

    func refresh() {
        DispatchQueue.global(qos: .utility).async { [weak self] in
            guard let self else { return }
            let output = Self.runLsof()
            let all = Self.parse(output)
            let filtered = all.filter {
                $0.port >= self.settings.portRangeMin && $0.port <= self.settings.portRangeMax
            }
            DispatchQueue.main.async { self.ports = filtered }
        }
    }

    @discardableResult
    func kill(entry: PortEntry) -> Bool {
        return Foundation.kill(pid_t(entry.pid), SIGTERM) == 0
    }

    static func runLsof() -> String {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/usr/sbin/lsof")
        process.arguments = ["-iTCP", "-sTCP:LISTEN", "-n", "-P"]
        let pipe = Pipe()
        process.standardOutput = pipe
        process.standardError = Pipe()
        try? process.run()
        process.waitUntilExit()
        return String(data: pipe.fileHandleForReading.readDataToEndOfFile(), encoding: .utf8) ?? ""
    }

    static func parse(_ output: String) -> [PortEntry] {
        var seen = Set<Int>()
        return output
            .components(separatedBy: "\n")
            .dropFirst()
            .compactMap { line -> PortEntry? in
                let parts = line.split(separator: " ", omittingEmptySubsequences: true)
                guard parts.count >= 9 else { return nil }
                let command = String(parts[0])
                guard let pid = Int(parts[1]) else { return nil }
                let name = String(parts[8])
                guard let portStr = name.split(separator: ":").last,
                      let port = Int(portStr) else { return nil }
                guard seen.insert(port).inserted else { return nil }
                return PortEntry(id: port, port: port, processName: command, pid: pid)
            }
            .sorted { $0.port < $1.port }
    }
}
