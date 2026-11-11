#!/bin/bash
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$DIR"

echo "=== 1. Building release binary ==="
~/.cargo/bin/cargo build --release

echo "=== 2. Creating macOS AppIcon.icns from ProctorX Logo.png ==="
ICONSET_DIR="/tmp/proctorx_icon.iconset"
rm -rf "$ICONSET_DIR"
mkdir -p "$ICONSET_DIR"

sips -z 16 16     "ProctorX Logo.png" --out "$ICONSET_DIR/icon_16x16.png" >/dev/null 2>&1
sips -z 32 32     "ProctorX Logo.png" --out "$ICONSET_DIR/icon_16x16@2x.png" >/dev/null 2>&1
sips -z 32 32     "ProctorX Logo.png" --out "$ICONSET_DIR/icon_32x32.png" >/dev/null 2>&1
sips -z 64 64     "ProctorX Logo.png" --out "$ICONSET_DIR/icon_32x32@2x.png" >/dev/null 2>&1
sips -z 128 128   "ProctorX Logo.png" --out "$ICONSET_DIR/icon_128x128.png" >/dev/null 2>&1
sips -z 256 256   "ProctorX Logo.png" --out "$ICONSET_DIR/icon_128x128@2x.png" >/dev/null 2>&1
sips -z 256 256   "ProctorX Logo.png" --out "$ICONSET_DIR/icon_256x256.png" >/dev/null 2>&1
sips -z 512 512   "ProctorX Logo.png" --out "$ICONSET_DIR/icon_256x256@2x.png" >/dev/null 2>&1
sips -z 512 512   "ProctorX Logo.png" --out "$ICONSET_DIR/icon_512x512.png" >/dev/null 2>&1
sips -z 1024 1024 "ProctorX Logo.png" --out "$ICONSET_DIR/icon_512x512@2x.png" >/dev/null 2>&1

mkdir -p assets
iconutil -c icns "$ICONSET_DIR" -o assets/AppIcon.icns
rm -rf "$ICONSET_DIR"

echo "=== 3. Assembling ProctorX.app bundle ==="
APP_BUNDLE="dist/ProctorX.app"
rm -rf "$APP_BUNDLE"
mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources"

cp target/release/contest_lockdown "$APP_BUNDLE/Contents/MacOS/contest_lockdown"
chmod +x "$APP_BUNDLE/Contents/MacOS/contest_lockdown"
cp assets/AppIcon.icns "$APP_BUNDLE/Contents/Resources/AppIcon.icns"

cat << 'EOF' > "$APP_BUNDLE/Contents/Info.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>contest_lockdown</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundleIdentifier</key>
    <string>com.proctorx.contestlockdown</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>ProctorX</string>
    <key>CFBundleDisplayName</key>
    <string>ProctorX</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
</dict>
</plist>
EOF

echo "=== 4. Packaging into ProctorX-v1.0.0-macOS.dmg ==="
DMG_ROOT="/tmp/proctorx_dmg_root"
rm -rf "$DMG_ROOT"
mkdir -p "$DMG_ROOT"

cp -R "$APP_BUNDLE" "$DMG_ROOT/"
ln -s /Applications "$DMG_ROOT/Applications"

DMG_OUT="dist/ProctorX-v1.0.0-macOS.dmg"
rm -f "$DMG_OUT"
hdiutil create -volname "ProctorX Installer" -srcfolder "$DMG_ROOT" -ov -format UDZO "$DMG_OUT"
rm -rf "$DMG_ROOT"

echo "=== SUCCESS! Created $DMG_OUT ==="
ls -lh "$DMG_OUT"
