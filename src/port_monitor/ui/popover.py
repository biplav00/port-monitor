"""Popover content: an inspector-style header (PORTS eyebrow + count pill +
refresh icon), a scrollable list of port rows, and a hint/quit footer.

Each row shows a status LED (green = your killable port, gray = another user's),
the port in SF Mono, the command, and an owner subtitle. The destructive Kill
control is revealed on hover. `render_ports_(ports, target)` rebuilds the list
and wires each Kill button to `target`'s `kill:` action (tag = pid)."""
from __future__ import annotations

import objc
from AppKit import (
    NSBezelStyleRegularSquare,
    NSBezierPath,
    NSBox,
    NSBoxSeparator,
    NSButton,
    NSColor,
    NSFont,
    NSFontAttributeName,
    NSFontWeightMedium,
    NSFontWeightRegular,
    NSFontWeightSemibold,
    NSForegroundColorAttributeName,
    NSImage,
    NSImageOnly,
    NSKernAttributeName,
    NSMakePoint,
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
from Foundation import NSAttributedString

_W = 380.0
_H = 460.0
_PAD = 16.0
_HEADER = 44.0
_FOOTER = 40.0
_ROW_H = 44.0
_GUTTER = 34.0  # x where text starts, leaving room for the LED


def _label(x, y, w, size, color, weight=NSFontWeightRegular, mono=False):
    f = NSTextField.alloc().initWithFrame_(NSMakeRect(x, y, w, size + 6))
    f.setBezeled_(False)
    f.setDrawsBackground_(False)
    f.setEditable_(False)
    f.setSelectable_(False)
    if mono:
        f.setFont_(NSFont.monospacedSystemFontOfSize_weight_(size, weight))
    else:
        f.setFont_(NSFont.systemFontOfSize_weight_(size, weight))
    f.setTextColor_(color)
    return f


def _tracked(field, text, kern):
    """Letter-spaced text — the typographic tell of an inspector eyebrow."""
    field.setAttributedStringValue_(
        NSAttributedString.alloc().initWithString_attributes_(
            text,
            {
                NSKernAttributeName: kern,
                NSFontAttributeName: field.font(),
                NSForegroundColorAttributeName: field.textColor(),
            },
        )
    )


def _icon_button(symbol, fallback, w=22.0):
    b = NSButton.alloc().initWithFrame_(NSMakeRect(0, 0, w, 22))
    img = NSImage.imageWithSystemSymbolName_accessibilityDescription_(symbol, None)
    if img is not None:
        img.setTemplate_(True)
        b.setImage_(img)
        b.setImagePosition_(NSImageOnly)
    else:
        b.setTitle_(fallback)
    b.setBordered_(False)
    b.setBezelStyle_(NSBezelStyleRegularSquare)
    b.setContentTintColor_(NSColor.secondaryLabelColor())
    return b


def _hline(x, y, w):
    box = NSBox.alloc().initWithFrame_(NSMakeRect(x, y, w, 1))
    box.setBoxType_(NSBoxSeparator)
    return box


class FlippedView(NSView):
    """Top-down coordinates so rows stack from the top of the scroll area."""

    def isFlipped(self):
        return True


class Row(NSView):
    """One port: status LED, port/command/owner, and a hover-revealed Kill."""

    @objc.python_method
    def build(self, p, target):
        self._hover = False
        self._mine = p.is_current_user

        self.port = _label(
            _GUTTER, 6, 56, 13, NSColor.labelColor(), NSFontWeightSemibold, mono=True
        )
        self.port.setStringValue_(f":{p.port}")
        self.name = _label(
            _GUTTER + 60, 6, _W - _GUTTER - 60 - 30, 13,
            NSColor.labelColor(), NSFontWeightMedium,
        )
        self.name.setStringValue_(p.command)
        self.sub = _label(
            _GUTTER + 60, 24, _W - _GUTTER - 60 - 30, 11, NSColor.secondaryLabelColor()
        )
        # Owner shown only when it isn't you — your own ports are the norm.
        self.sub.setStringValue_(
            f"pid {p.pid}" if p.is_current_user else f"pid {p.pid} · {p.user}"
        )

        for v in (self.port, self.name, self.sub):
            self.addSubview_(v)

        # Kill is destructive: red, icon-only, hidden until the row is hovered.
        # Another user's process can't be killed, so it gets no control at all.
        if p.is_current_user:
            self.kill = _icon_button("xmark.circle.fill", "✕")
            self.kill.setContentTintColor_(NSColor.systemRedColor())
            self.kill.setFrame_(NSMakeRect(_W - _PAD - 22, 11, 22, 22))
            self.kill.setTag_(p.pid)
            self.kill.setTarget_(target)
            self.kill.setAction_("kill:")
            self.kill.setHidden_(True)
            self.addSubview_(self.kill)
        else:
            for v in (self.port, self.name, self.sub):
                v.setAlphaValue_(0.5)
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
        if self._mine:
            self.kill.setHidden_(False)
        self.setNeedsDisplay_(True)

    def mouseExited_(self, _e):
        self._hover = False
        if self._mine:
            self.kill.setHidden_(True)
        self.setNeedsDisplay_(True)

    def drawRect_(self, _rect):
        b = self.bounds()
        if self._hover:
            inset = NSMakeRect(8, 3, b.size.width - 16, b.size.height - 6)
            NSColor.colorWithWhite_alpha_(0.5, 0.12).set()
            NSBezierPath.bezierPathWithRoundedRect_xRadius_yRadius_(inset, 7, 7).fill()

        # Status LED — the one bit of color: green = yours/live, gray = not.
        r = 3.5
        led = NSMakeRect(_PAD, 15 - r, 2 * r, 2 * r)
        color = NSColor.systemGreenColor() if self._mine else NSColor.tertiaryLabelColor()
        color.set()
        NSBezierPath.bezierPathWithOvalInRect_(led).fill()


class PopoverView(NSView):
    @objc.python_method
    def build(self):
        self.setFrame_(NSMakeRect(0, 0, _W, _H))

        # Header: tracked PORTS eyebrow, count pill, refresh icon.
        self.title = _label(
            _PAD, _H - 30, 60, 11, NSColor.secondaryLabelColor(), NSFontWeightSemibold
        )
        _tracked(self.title, "PORTS", 1.4)

        self.count = _label(
            _PAD + 52, _H - 31, 40, 11, NSColor.secondaryLabelColor(), NSFontWeightSemibold
        )
        self.count.setAlignment_(NSTextAlignmentCenter)
        self.count.setWantsLayer_(True)
        self.count.layer().setCornerRadius_(8.0)
        self.count.layer().setBackgroundColor_(
            NSColor.colorWithWhite_alpha_(0.5, 0.15).CGColor()
        )

        self.refresh = _icon_button("arrow.clockwise", "↻")
        self.refresh.setFrame_(NSMakeRect(_W - _PAD - 22, _H - 32, 22, 22))

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

        # Footer: surface the otherwise-hidden force-kill shortcut, plus Quit.
        footer_sep = _hline(0, _FOOTER - 1, _W)
        self.hint = _label(
            _PAD, 11, 200, 11, NSColor.tertiaryLabelColor()
        )
        self.hint.setStringValue_("Hold ⇧ to force kill")
        self.quit = _icon_button("power", "Quit", w=44)
        self.quit.setFrame_(NSMakeRect(_W - _PAD - 22, 9, 22, 22))

        for v in (
            self.title, self.count, self.refresh, header_sep, self.scroll,
            self.empty, footer_sep, self.hint, self.quit,
        ):
            self.addSubview_(v)
        return self

    @objc.python_method
    def render_ports_(self, ports, target):
        self.count.setStringValue_(str(len(ports)))
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
