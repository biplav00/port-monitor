#!/usr/bin/env bash
# Build "Port Monitor.app" with PyInstaller, then package it for distribution.
#
# Outputs:
#   packaging/dist/Port Monitor.app   — the menu-bar app bundle
#   packaging/dist/Port Monitor.dmg   — drag-to-install disk image (UDZO)
#   packaging/dist/Port Monitor.pkg   — flat installer package (component)
#
# Usage: packaging/build_dmg.sh
set -euo pipefail

cd "$(dirname "$0")"   # packaging/

echo "==> Cleaning previous build"
rm -rf build dist

echo "==> Building .app with PyInstaller"
uv run pyinstaller --noconfirm port-monitor.spec

APP="dist/Port Monitor.app"
DMG="dist/Port Monitor.dmg"
PKG="dist/Port Monitor.pkg"
[ -d "$APP" ] || { echo "ERROR: $APP not produced"; exit 1; }

# ---- DMG (drag-to-install) -----------------------------------------------
echo "==> Staging .dmg contents"
STAGE="$(mktemp -d)"
cp -R "$APP" "$STAGE/"
ln -s /Applications "$STAGE/Applications"   # drag-to-install target

echo "==> Creating $DMG"
rm -f "$DMG"
hdiutil create -volname "Port Monitor" -srcfolder "$STAGE" -ov -format UDZO "$DMG" >/dev/null
rm -rf "$STAGE"
ls -lh "$DMG"

# ---- PKG (component installer) -------------------------------------------
# A flat .pkg installs the .app into /Applications without prompting the user.
# Use pkgbuild for component packages; productbuild for a full distribution
# (we keep it simple — component is enough for an unsigned dev build).
echo "==> Building component .pkg"
rm -rf pkgroot pkg-component.plist
mkdir -p pkgroot
cp -R "$APP" pkgroot/

# pkgbuild wants a component plist describing install rules. The defaults
# (install to /Applications) work, but we set bundle identifier explicitly
# so future signed/notarized builds have a stable identity.
cat > pkg-component.plist <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<array>
  <dict>
    <key>BundleIsRelocatable</key>
    <true/>
    <key>BundleIsVersionChecked</key>
    <true/>
    <key>BundleOverwriteAction</key>
    <string>upgrade</string>
    <key>RootRelativeBundlePath</key>
    <string>Port Monitor.app</string>
  </dict>
</array>
</plist>
PLIST

VERSION="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$APP/Contents/Info.plist")"
BUNDLE_ID="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$APP/Contents/Info.plist")"

pkgbuild \
  --root pkgroot \
  --component-plist pkg-component.plist \
  --identifier "$BUNDLE_ID" \
  --version "$VERSION" \
  --install-location /Applications \
  "$PKG"

rm -rf pkgroot pkg-component.plist
ls -lh "$PKG"

echo "==> Done"