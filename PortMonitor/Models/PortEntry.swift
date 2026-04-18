import Foundation

struct PortEntry: Identifiable, Equatable {
    let id: Int
    let port: Int
    let processName: String
    let pid: Int
}
