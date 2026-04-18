import SwiftUI

final class AppSettings: ObservableObject {
    static let shared = AppSettings()

    @AppStorage("refreshInterval") var refreshInterval: Double = 3.0
    @AppStorage("portRangeMin")    var portRangeMin: Int = 1024
    @AppStorage("portRangeMax")    var portRangeMax: Int = 65535
}
