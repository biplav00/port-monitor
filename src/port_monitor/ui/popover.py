"""Popover content: a header, a scrollable list of port rows (each with a Kill
button), and a footer. `render_ports_(ports, target)` rebuilds the list and wires
each Kill button to `target`'s `kill:` action (button tag = pid)."""
from __future__ import annotations

import objc
from AppKit import (
    NSBezelStyleRounded,
    NSColor,
    NSFont,
    NSFontWeightRegular,
    NSMakeRect,
    NSNoBorder,
    NSScrollView,
    NSTextAlignmentRight,
    NSTextField,
    NSView,
)

_W = 360.0
_H = 460.0
_PAD = 14.0
_HEADER = 46.0
_FOOTER = 42.0
_ROW_H = 46.0


def _label(x, y, w, size, color, bold=False, mono=False):
    f = NSTextField.alloc().initWithFrame_(NSMakeRect(x, y, w, size + 8))
    f.setBezeled_(False)
    f.setDrawsBackground_(False)
    f.setEditable_(False)
    f.setSelectable_(False)
    if mono:
        f.setFont_(NSFont.monospacedSystemFontOfSize_weight_(size, NSFontWeightRegular))
    else:
        f.setFont_(
            NSFont.boldSystemFontOfSize_(size) if bold else NSFont.systemFontOfSize_(size)
        )
    f.setTextColor_(color)
    return f


class FlippedView(NSView):
    """Top-down coordinates so rows stack from the top of the scroll area."""

    def isFlipped(self):
        return True


class PopoverView(NSView):
    @objc.python_method
    def build(self):
        self.setFrame_(NSMakeRect(0, 0, _W, _H))
        iw = _W - 2 * _PAD

        # Header.
        self.title = _label(_PAD, _H - 34, 120, 15, NSColor.labelColor(), bold=True)
        self.title.setStringValue_("Ports")
        self.count = _label(
            _PAD + 60, _H - 33, 80, 12, NSColor.secondaryLabelColor(), bold=True
        )

        # Scrollable list.
        scroll_h = _H - _HEADER - _FOOTER
        self.scroll = NSScrollView.alloc().initWithFrame_(
            NSMakeRect(0, _FOOTER, _W, scroll_h)
        )
        self.scroll.setHasVerticalScroller_(True)
        self.scroll.setDrawsBackground_(False)
        self.scroll.setBorderType_(NSNoBorder)
        self.doc = FlippedView.alloc().initWithFrame_(NSMakeRect(0, 0, _W, scroll_h))
        self.scroll.setDocumentView_(self.doc)

        self.empty = _label(
            0, scroll_h / 2, _W, 13, NSColor.tertiaryLabelColor()
        )
        self.empty.setStringValue_("No listening ports")
        self.empty.setAlignment_(NSTextAlignmentRight)
        self.empty.setFrame_(NSMakeRect(0, _FOOTER + scroll_h / 2, _W, 20))
        self.empty.setAlignment_(1)  # center

        # Footer.
        self.refresh = self._button(_W - _PAD - 150, 9, 70, "Refresh")
        self.quit = self._button(_W - _PAD - 74, 9, 70, "Quit")

        for v in (self.title, self.count, self.scroll, self.empty, self.refresh, self.quit):
            self.addSubview_(v)
        return self

    @objc.python_method
    def _button(self, x, y, w, title):
        from AppKit import NSButton

        b = NSButton.alloc().initWithFrame_(NSMakeRect(x, y, w, 24))
        b.setTitle_(title)
        b.setBezelStyle_(NSBezelStyleRounded)
        return b

    @objc.python_method
    def render_ports_(self, ports, target):
        self.count.setStringValue_(f"({len(ports)})")
        self.empty.setHidden_(bool(ports))

        # Rebuild rows.
        for sub in list(self.doc.subviews()):
            sub.removeFromSuperview()

        from AppKit import NSButton

        iw = _W - 2 * _PAD
        height = max(len(ports) * _ROW_H, self.scroll.frame().size.height)
        self.doc.setFrame_(NSMakeRect(0, 0, _W, height))

        for i, p in enumerate(ports):
            y = i * _ROW_H
            port = _label(
                _PAD, y + 13, 64, 13, NSColor.systemIndigoColor(), bold=True, mono=True
            )
            port.setStringValue_(f":{p.port}")
            name = _label(_PAD + 70, y + 22, iw - 150, 13, NSColor.labelColor(), bold=True)
            name.setStringValue_(p.command)
            sub = _label(
                _PAD + 70, y + 6, iw - 150, 11, NSColor.secondaryLabelColor()
            )
            # Show the owner only when it isn't you — your own ports are the norm.
            sub.setStringValue_(
                f"pid {p.pid}" if p.is_current_user else f"pid {p.pid} · {p.user}"
            )

            kill = NSButton.alloc().initWithFrame_(NSMakeRect(_W - _PAD - 60, y + 11, 60, 24))
            kill.setTitle_("Kill")
            kill.setBezelStyle_(NSBezelStyleRounded)
            kill.setTag_(p.pid)
            kill.setTarget_(target)
            kill.setAction_("kill:")
            # Can't kill another user's process without privileges: dim + disable.
            if not p.is_current_user:
                kill.setEnabled_(False)
                for v in (port, name, sub):
                    v.setAlphaValue_(0.55)

            for v in (port, name, sub, kill):
                self.doc.addSubview_(v)
