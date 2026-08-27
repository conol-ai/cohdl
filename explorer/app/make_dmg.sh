#!/bin/sh
# Assemble CoHDL Explorer.app and wrap it in a drag-install DMG.
#
#   make_dmg.sh VERSION DIST_DIR OUT_DMG BINARY [BINARY...]
#
#   VERSION   X.Y.Z, stamped into Info.plist
#   DIST_DIR  the built web frontend (explorer/web/dist)
#   OUT_DMG   output path
#   BINARY    one cohdl-explorer per architecture; two are lipo'd universal
#
# Signing is controlled by the environment: SIGN_IDENTITY names the
# Developer ID Application identity (unset = unsigned local build), and
# SIGN_KEYCHAIN optionally points codesign at a specific keychain. The app
# is signed with hardened runtime, then the DMG itself is signed too.
# Notarization/stapling is the caller's job (the release workflow) — this
# script is fully offline apart from codesign's secure timestamp.
set -eu

[ $# -ge 4 ] || { echo "usage: make_dmg.sh VERSION DIST_DIR OUT_DMG BINARY..." >&2; exit 2; }
version="$1"; dist="$2"; out="$3"; shift 3

here="$(cd "$(dirname "$0")" && pwd)"
stage="$(mktemp -d)"
trap 'rm -rf "$stage"' EXIT

app="$stage/root/CoHDL Explorer.app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"

if [ $# -gt 1 ]; then
  lipo -create "$@" -output "$app/Contents/MacOS/cohdl-explorer"
else
  cp "$1" "$app/Contents/MacOS/cohdl-explorer"
fi
chmod 755 "$app/Contents/MacOS/cohdl-explorer"

sed "s/@VERSION@/$version/g" "$here/Info.plist" > "$app/Contents/Info.plist"
cp "$here/icon.icns" "$app/Contents/Resources/icon.icns"
cp -R "$dist" "$app/Contents/Resources/web"

if [ -n "${SIGN_IDENTITY:-}" ]; then
  # $@ (the input binaries) is spent, so rebuild it as codesign's flag list.
  set -- --force --timestamp
  [ -n "${SIGN_KEYCHAIN:-}" ] && set -- "$@" --keychain "$SIGN_KEYCHAIN"
  codesign "$@" --options runtime --sign "$SIGN_IDENTITY" "$app"
  codesign --verify --strict --deep -v "$app"
fi

ln -s /Applications "$stage/root/Applications"
hdiutil create -volname "CoHDL Explorer" -srcfolder "$stage/root" \
  -ov -format UDZO "$out" >/dev/null

if [ -n "${SIGN_IDENTITY:-}" ]; then
  codesign "$@" --sign "$SIGN_IDENTITY" "$out"
fi

echo "built $out"
