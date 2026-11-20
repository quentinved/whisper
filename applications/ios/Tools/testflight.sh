#!/usr/bin/env bash
# Archive the app and upload it to App Store Connect, where it lands in
# TestFlight (groups with automatic distribution get it once processed).
#
# Needs Xcode signed in to an Apple account on team 7WBNUC3KR9
# (Xcode → Settings → Accounts) and an App Store Connect app record for
# com.quentinvedrenne.whisper.
#
#   applications/ios/Tools/testflight.sh
set -euo pipefail

IOS="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$(mktemp -d)"
trap 'rm -rf "$OUT"' EXIT

# Same yymmdd.HHMM scheme as the team's other apps, so every upload is unique
# without bumping CURRENT_PROJECT_VERSION in the project by hand.
BUILD="$(date +%y%m%d.%H%M)"

cat > "$OUT/ExportOptions.plist" <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>method</key><string>app-store-connect</string>
	<key>destination</key><string>upload</string>
	<key>teamID</key><string>7WBNUC3KR9</string>
	<key>signingStyle</key><string>automatic</string>
	<key>uploadSymbols</key><true/>
	<key>manageAppVersionAndBuildNumber</key><false/>
</dict>
</plist>
EOF

echo "==> Archiving build $BUILD"
xcodebuild archive -quiet \
  -project "$IOS/Whisper.xcodeproj" -scheme Whisper -configuration Release \
  -destination 'generic/platform=iOS' -archivePath "$OUT/Whisper.xcarchive" \
  -allowProvisioningUpdates CURRENT_PROJECT_VERSION="$BUILD"

echo "==> Uploading to App Store Connect"
xcodebuild -exportArchive \
  -archivePath "$OUT/Whisper.xcarchive" -exportPath "$OUT/export" \
  -exportOptionsPlist "$OUT/ExportOptions.plist" -allowProvisioningUpdates

echo "Uploaded build $BUILD. It shows in TestFlight once Apple finishes processing (usually 5–15 min)."
