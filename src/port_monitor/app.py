"""Native macOS menu-bar app: an NSStatusItem toggling an NSPopover that lists
listening TCP ports, each with a Kill button. NSPopover floats over full-screen
apps and dismisses on click-away for free."""
from __future__ import annotations

import objc
from AppKit import (
    NSApplication,
    NSApplicationActivationPolicyAccessory,
    NSEvent,
    NSEventModifierFlagShift,
    NSImage,
    NSMakeRect,
    NSMinYEdge,
    NSObject,
    NSPopover,
    NSPopoverBehaviorTransient,
    NSStatusBar,
    NSTimer,
    NSVariableStatusItemLength,
    NSViewController,
)

from . import ports
from .ui.popover import PopoverView

_POLL = 3.0  # seconds


class _Controller(NSObject):
    def init(self):
        self = objc.super(_Controller, self).init()
        if self is None:
            return None

        bar = NSStatusBar.systemStatusBar()
        self.item = bar.statusItemWithLength_(NSVariableStatusItemLength)
        btn = self.item.button()
        img = NSImage.imageWithSystemSymbolName_accessibilityDescription_(
            "dot.radiowaves.up.forward", "Port Monitor"
        )
        if img is not None:
            img.setTemplate_(True)
            btn.setImage_(img)
        else:
            btn.setTitle_("⦿")
        btn.setTarget_(self)
        btn.setAction_("toggle:")

        self.view = (
            PopoverView.alloc().initWithFrame_(NSMakeRect(0, 0, 360, 460)).build()
        )
        vc = NSViewController.alloc().init()
        vc.setView_(self.view)
        self.popover = NSPopover.alloc().init()
        self.popover.setContentViewController_(vc)
        self.popover.setBehavior_(NSPopoverBehaviorTransient)

        self.view.refresh.setTarget_(self)
        self.view.refresh.setAction_("refresh:")
        self.view.quit.setTarget_(self)
        self.view.quit.setAction_("quitApp:")

        self._refresh()
        NSTimer.scheduledTimerWithTimeInterval_target_selector_userInfo_repeats_(
            _POLL, self, "tick:", None, True
        )
        return self

    @objc.python_method
    def _refresh(self):
        try:
            self.view.render_ports_(ports.list_listening(), self)
        except Exception:
            pass  # never let a render error kill the timer loop

    def tick_(self, _timer):
        self._refresh()

    def refresh_(self, _sender):
        self._refresh()

    def kill_(self, sender):
        force = bool(NSEvent.modifierFlags() & NSEventModifierFlagShift)
        try:
            ports.kill(int(sender.tag()), force)
        except (ProcessLookupError, PermissionError):
            pass
        self._refresh()

    def toggle_(self, sender):
        if self.popover.isShown():
            self.popover.performClose_(sender)
        else:
            self._refresh()
            self.popover.showRelativeToRect_ofView_preferredEdge_(
                sender.bounds(), sender, NSMinYEdge
            )

    def quitApp_(self, _sender):
        NSApplication.sharedApplication().terminate_(None)


_RETAIN = []  # keep the controller (and its status item) alive


def run():
    app = NSApplication.sharedApplication()
    app.setActivationPolicy_(NSApplicationActivationPolicyAccessory)
    controller = _Controller.alloc().init()
    _RETAIN.append(controller)
    app.run()
