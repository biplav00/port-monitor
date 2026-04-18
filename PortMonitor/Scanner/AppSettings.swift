import SwiftUI

enum AppearanceMode: String, CaseIterable {
    case system, light, dark

    var label: String {
        switch self {
        case .system: return "System"
        case .light:  return "Light"
        case .dark:   return "Dark"
        }
    }
}

final class AppSettings: ObservableObject {
    static let shared = AppSettings()

    @AppStorage("refreshInterval") var refreshInterval: Double = 3.0
    @AppStorage("portRangeMin")    var portRangeMin: Int = 1024
    @AppStorage("portRangeMax")    var portRangeMax: Int = 65535

    @Published var appearance: AppearanceMode {
        didSet { UserDefaults.standard.set(appearance.rawValue, forKey: "appearance") }
    }

    private init() {
        let raw = UserDefaults.standard.string(forKey: "appearance") ?? AppearanceMode.system.rawValue
        self.appearance = AppearanceMode(rawValue: raw) ?? .system
    }
}
