# PyInstaller spec for port-monitor — macOS menu-bar app (.app bundle).
# Run from the packaging/ dir:  uv run pyinstaller port-monitor.spec
# (or use packaging/build_dmg.sh). Paths below are relative to this dir.
from PyInstaller.utils.hooks import collect_all

# Pull the pyobjc AppKit/objc bits so the bundle is self-contained.
ak_d, ak_b, ak_h = collect_all("AppKit")
oc_d, oc_b, oc_h = collect_all("objc")
fn_d, fn_b, fn_h = collect_all("Foundation")
pkg_datas = ak_d + oc_d + fn_d
pkg_binaries = ak_b + oc_b + fn_b
pkg_hidden = ak_h + oc_h + fn_h + [
    "port_monitor",
    "port_monitor.app",
    "port_monitor.ports",
    "port_monitor.types",
    "port_monitor.ui.popover",
]

a = Analysis(
    ["launcher.py"],
    pathex=["../src"],          # so `import port_monitor` resolves from source
    binaries=pkg_binaries,
    datas=pkg_datas,
    hiddenimports=pkg_hidden,
    hookspath=[],
    runtime_hooks=[],
    excludes=[],
)
pyz = PYZ(a.pure)

exe = EXE(
    pyz,
    a.scripts,
    [],
    exclude_binaries=True,
    name="Port Monitor",        # CFBundleExecutable → proper-cased process name
    console=False,              # GUI / windowed
)

coll = COLLECT(exe, a.binaries, a.datas, name="Port Monitor")

app = BUNDLE(
    coll,
    name="Port Monitor.app",
    icon="port-monitor.icns",
    bundle_identifier="com.portmonitor.PortMonitor",
    info_plist={
        "LSUIElement": True,    # menu-bar only: no Dock icon, no window
        "CFBundleName": "Port Monitor",
        "CFBundleDisplayName": "Port Monitor",
        "CFBundleShortVersionString": "0.3.0",
        "CFBundleVersion": "0.3.0",
        "LSMinimumSystemVersion": "12.0",
    },
)
