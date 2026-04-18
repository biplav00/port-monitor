import AppKit
import SwiftUI
import Combine

final class AppDelegate: NSObject, NSApplicationDelegate {

    private var statusItem: NSStatusItem!
    private var popover: NSPopover!
    private var cancellables = Set<AnyCancellable>()

    let scanner = PortScanner()
    let settings = AppSettings.shared

    func applicationDidFinishLaunching(_ notification: Notification) {
        setupStatusItem()
        setupPopover()
        scanner.start()
        observePortCount()
        observeAppearance()
    }

    private func setupStatusItem() {
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        guard let button = statusItem.button else { return }
        button.image = NSImage(systemSymbolName: "network", accessibilityDescription: "Port Monitor")
        button.imagePosition = .imageLeading
        button.action = #selector(togglePopover)
        button.target = self
    }

    private func setupPopover() {
        let content = PopoverView()
            .environmentObject(scanner)
            .environmentObject(settings)

        popover = NSPopover()
        popover.contentSize = NSSize(width: 300, height: 200)
        popover.behavior = .transient
        popover.contentViewController = NSHostingController(rootView: content)
        applyAppearance(settings.appearance)
    }

    private func observePortCount() {
        scanner.$ports
            .receive(on: RunLoop.main)
            .sink { [weak self] ports in
                let title = ports.isEmpty ? "" : "  \(ports.count)"
                self?.statusItem.button?.title = title
            }
            .store(in: &cancellables)
    }

    private func observeAppearance() {
        settings.$appearance
            .receive(on: RunLoop.main)
            .sink { [weak self] mode in self?.applyAppearance(mode) }
            .store(in: &cancellables)
    }

    private func applyAppearance(_ mode: AppearanceMode) {
        switch mode {
        case .system: popover.appearance = nil
        case .light:  popover.appearance = NSAppearance(named: .aqua)
        case .dark:   popover.appearance = NSAppearance(named: .darkAqua)
        }
    }

    @objc private func togglePopover(_ sender: AnyObject?) {
        guard let button = statusItem.button else { return }
        if popover.isShown {
            popover.performClose(nil)
        } else {
            popover.show(relativeTo: button.bounds, of: button, preferredEdge: .minY)
            popover.contentViewController?.view.window?.makeKey()
        }
    }
}
