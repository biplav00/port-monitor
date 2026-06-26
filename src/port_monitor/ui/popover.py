"""Popover content: header, a scrollable list of port rows (hover-highlighted,
each with a Kill button), and a footer. `render_ports_(ports, target)` rebuilds
the list and wires each Kill button to `target`'s `kill:` action (tag = pid)."""
from __future__ import annotations

import objc
from AppKit import (
    NSBezelStyleRounded,
    NSBezierPath,
    NSBox,
    NSBoxSeparator,
    NSButton,
    NSColor,
    NSFont,
    NSFontWeightRegular,
    NSMakeRect,
    NSNoBorder,
    NSScrollView,
    NSTextAlignmentCenter,
    NSTextField,
    NSTrackingActiveInKeyWindow,
    NSTrackingArea,
    NSTrackingInVisibleRect,
    NSTrackingMouseEnteredAndExited,
    NSView,
)

_W = 360.0
_H = 460.0
_PAD = 14.0
_HEADER = 46.0
_FOOTER = 42.0
_ROW_H = 48.0


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


def _hline(x, y, w):
    box = NSBox.alloc().initWithFrame_(NSMakeRect(x, y, w, 1))
    box.setBoxType_(NSBoxSeparator)
    return box


class FlippedView(NSView):
    """Top-down coordinates so rows stack from the top of the scroll area."""

    def isFlipped(self):
        return True


class Row(NSView):
    """One port: hover-highlighted, with port/name/owner and a Kill button."""

    @objc.python_method
    def build(self, p, target):
        self._hover = False
        iw = _W - 2 * _PAD

        self.port = _label(
            _PAD, 15, 64, 13, NSColor.systemIndigoColor(), bold=True, mono=True
        )
        self.port.setStringValue_(f":{p.port}")
        self.name = _label(_PAD + 70, 25, iw - 150, 13, NSColor.labelColor(), bold=True)
        self.name.setStringValue_(p.command)
        self.sub = _label(_PAD + 70, 9, iw - 150, 11, NSColor.secondaryLabelColor())
        # Owner shown only when it isn't you — your own ports are the norm.
        self.sub.setStringValue_(
            f"pid {p.pid}" if p.is_current_user else f"pid {p.pid} · {p.user}"
        )

        self.kill = NSButton.alloc().initWithFrame_(NSMakeRect(_W - _PAD - 60, 13, 60, 24))
        self.kill.setTitle_("Kill")
        self.kill.setBezelStyle_(NSBezelStyleRounded)
        self.kill.setTag_(p.pid)
        self.kill.setTarget_(target)
        self.kill.setAction_("kill:")
        if not p.is_current_user:  # can't kill another user's process
            self.kill.setEnabled_(False)
            for v in (self.port, self.name, self.sub):
                v.setAlphaValue_(0.55)

        for v in (self.port, self.name, self.sub, self.kill):
            self.addSubview_(v)
        return self

    def isFlipped(self):
        return True

    def updateTrackingAreas(self):
        for a in list(self.trackingAreas()):
            self.removeTrackingArea_(a)
        area = NSTrackingArea.alloc().initWithRect_options_owner_userInfo_(
            self.bounds(),
            NSTrackingMouseEnteredAndExited
            | NSTrackingActiveInKeyWindow
            | NSTrackingInVisibleRect,
            self,
            None,
        )
        self.addTrackingArea_(area)

    def mouseEntered_(self, _e):
        self._hover = True
        self.setNeedsDisplay_(True)

    def mouseExited_(self, _e):
        self._hover = False
        self.setNeedsDisplay_(True)

    def drawRect_(self, _rect):
        if not self._hover:
            return
        b = self.bounds()
        inset = NSMakeRect(b.origin.x + 7, b.origin.y + 3, b.size.width - 14, b.size.height - 6)
        NSColor.colorWithWhite_alpha_(0.5, 0.13).set()
        NSBezierPath.bezierPathWithRoundedRect_xRadius_yRadius_(inset, 8, 8).fill()


class PopoverView(NSView):
    @objc.python_method
    def build(self):
        self.setFrame_(NSMakeRect(0, 0, _W, _H))

        # Header.
        self.title = _label(_PAD, _H - 34, 120, 15, NSColor.labelColor(), bold=True)
        self.title.setStringValue_("Ports")
        self.count = _label(
            _PAD + 56, _H - 33, 80, 12, NSColor.secondaryLabelColor(), bold=True
        )
        header_sep = _hline(0, _H - _HEADER, _W)

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
            0, _FOOTER + scroll_h / 2, _W, 13, NSColor.tertiaryLabelColor()
        )
        self.empty.setStringValue_("No listening ports")
        self.empty.setAlignment_(NSTextAlignmentCenter)

        # Footer.
        footer_sep = _hline(0, _FOOTER - 1, _W)
        self.refresh = self._button(_W - _PAD - 150, 9, 70, "Refresh")
        self.quit = self._button(_W - _PAD - 74, 9, 70, "Quit")

        for v in (
            self.title, self.count, header_sep, self.scroll, self.empty,
            footer_sep, self.refresh, self.quit,
        ):
            self.addSubview_(v)
        return self

    @objc.python_method
    def _button(self, x, y, w, title):
        b = NSButton.alloc().initWithFrame_(NSMakeRect(x, y, w, 24))
        b.setTitle_(title)
        b.setBezelStyle_(NSBezelStyleRounded)
        return b

    @objc.python_method
    def render_ports_(self, ports, target):
        self.count.setStringValue_(f"({len(ports)})")
        self.empty.setHidden_(bool(ports))

        for sub in list(self.doc.subviews()):
            sub.removeFromSuperview()

        height = max(len(ports) * _ROW_H, self.scroll.frame().size.height)
        self.doc.setFrame_(NSMakeRect(0, 0, _W, height))

        for i, p in enumerate(ports):
            row = Row.alloc().initWithFrame_(
                NSMakeRect(0, i * _ROW_H, _W, _ROW_H)
            ).build(p, target)
            self.doc.addSubview_(row)
