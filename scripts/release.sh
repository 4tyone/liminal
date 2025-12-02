#!/bin/bash

# Liminal Release Script
# Usage: ./scripts/release.sh <version>
# Example: ./scripts/release.sh 0.2.0

set -e

VERSION=$1
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
OUTPUT_DIR="$PROJECT_DIR/releases/$VERSION"
PRIVATE_KEY_PATH="$HOME/.tauri/liminal.key"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
echo_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
echo_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Check version argument
if [ -z "$VERSION" ]; then
    echo_error "Version argument required"
    echo "Usage: ./scripts/release.sh <version>"
    echo "Example: ./scripts/release.sh 0.2.0"
    exit 1
fi

# Validate version format
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo_error "Invalid version format. Use semantic versioning (e.g., 0.2.0)"
    exit 1
fi

# Check for private key
if [ ! -f "$PRIVATE_KEY_PATH" ]; then
    echo_error "Private key not found at $PRIVATE_KEY_PATH"
    echo "Generate one with: npx @tauri-apps/cli signer generate -w ~/.tauri/liminal.key"
    exit 1
fi

echo_info "Starting release process for version $VERSION"

# Update version in tauri.conf.json
echo_info "Updating version in tauri.conf.json..."
cd "$PROJECT_DIR"
sed -i '' "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" src-tauri/tauri.conf.json

# Update version in Cargo.toml
echo_info "Updating version in Cargo.toml..."
sed -i '' "s/^version = \"[^\"]*\"/version = \"$VERSION\"/" src-tauri/Cargo.toml

# Set signing environment variables
export TAURI_SIGNING_PRIVATE_KEY="$(cat "$PRIVATE_KEY_PATH")"
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD="Liminal"

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Build for Apple Silicon
echo_info "Building Apple Silicon..."
npm run tauri build -- --target aarch64-apple-darwin

# Re-sign the app
echo_info "Re-signing Apple Silicon app..."
AARCH64_APP="src-tauri/target/aarch64-apple-darwin/release/bundle/macos/Liminal.app"
codesign --force --deep --sign - "$AARCH64_APP"

# Create DMG with signed app
echo_info "Creating Apple Silicon DMG..."
AARCH64_DMG="src-tauri/target/aarch64-apple-darwin/release/bundle/dmg/Liminal_${VERSION}_aarch64.dmg"
DMG_TMP="$PROJECT_DIR/.dmg-staging"
rm -rf "$DMG_TMP"
mkdir -p "$DMG_TMP"
cp -R "$AARCH64_APP" "$DMG_TMP/"
ln -s /Applications "$DMG_TMP/Applications"
rm -f "$AARCH64_DMG"
hdiutil create -volname "Liminal" -srcfolder "$DMG_TMP" -ov -format UDZO "$AARCH64_DMG"
rm -rf "$DMG_TMP"

# Build for Intel
echo_info "Building Intel..."
npm run tauri build -- --target x86_64-apple-darwin

# Re-sign the app
echo_info "Re-signing Intel app..."
X64_APP="src-tauri/target/x86_64-apple-darwin/release/bundle/macos/Liminal.app"
codesign --force --deep --sign - "$X64_APP"

# Create DMG with signed app
echo_info "Creating Intel DMG..."
X64_DMG="src-tauri/target/x86_64-apple-darwin/release/bundle/dmg/Liminal_${VERSION}_x64.dmg"
rm -rf "$DMG_TMP"
mkdir -p "$DMG_TMP"
cp -R "$X64_APP" "$DMG_TMP/"
ln -s /Applications "$DMG_TMP/Applications"
rm -f "$X64_DMG"
hdiutil create -volname "Liminal" -srcfolder "$DMG_TMP" -ov -format UDZO "$X64_DMG"
rm -rf "$DMG_TMP"

# Copy artifacts
echo_info "Copying artifacts to $OUTPUT_DIR..."

# Apple Silicon artifacts
AARCH64_BUNDLE="src-tauri/target/aarch64-apple-darwin/release/bundle"
cp "$AARCH64_BUNDLE/dmg/Liminal_${VERSION}_aarch64.dmg" "$OUTPUT_DIR/"
cp "$AARCH64_BUNDLE/macos/Liminal.app.tar.gz" "$OUTPUT_DIR/Liminal_${VERSION}_aarch64.app.tar.gz"
cp "$AARCH64_BUNDLE/macos/Liminal.app.tar.gz.sig" "$OUTPUT_DIR/Liminal_${VERSION}_aarch64.app.tar.gz.sig"

# Intel artifacts
X64_BUNDLE="src-tauri/target/x86_64-apple-darwin/release/bundle"
cp "$X64_BUNDLE/dmg/Liminal_${VERSION}_x64.dmg" "$OUTPUT_DIR/"
cp "$X64_BUNDLE/macos/Liminal.app.tar.gz" "$OUTPUT_DIR/Liminal_${VERSION}_x64.app.tar.gz"
cp "$X64_BUNDLE/macos/Liminal.app.tar.gz.sig" "$OUTPUT_DIR/Liminal_${VERSION}_x64.app.tar.gz.sig"

# Read signatures
AARCH64_SIG=$(cat "$OUTPUT_DIR/Liminal_${VERSION}_aarch64.app.tar.gz.sig")
X64_SIG=$(cat "$OUTPUT_DIR/Liminal_${VERSION}_x64.app.tar.gz.sig")

# Get current date in ISO format
PUB_DATE=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Generate latest.json
echo_info "Generating latest.json..."
cat > "$OUTPUT_DIR/latest.json" << EOF
{
  "version": "$VERSION",
  "notes": "Liminal v$VERSION",
  "pub_date": "$PUB_DATE",
  "platforms": {
    "darwin-aarch64": {
      "signature": "$AARCH64_SIG",
      "url": "https://github.com/4tyone/liminal-download/releases/download/$VERSION/Liminal_${VERSION}_aarch64.app.tar.gz"
    },
    "darwin-x86_64": {
      "signature": "$X64_SIG",
      "url": "https://github.com/4tyone/liminal-download/releases/download/$VERSION/Liminal_${VERSION}_x64.app.tar.gz"
    }
  }
}
EOF

echo_info "Release artifacts created in $OUTPUT_DIR:"
ls -la "$OUTPUT_DIR"

# GitHub Release
GITHUB_REPO="4tyone/liminal-download"

echo ""
echo_info "Creating GitHub release for tag $VERSION..."

# Check if gh is installed
if ! command -v gh &> /dev/null; then
    echo_error "GitHub CLI (gh) is not installed. Please install it with: brew install gh"
    exit 1
fi

# Check if authenticated
if ! gh auth status &> /dev/null; then
    echo_error "Not authenticated with GitHub CLI. Please run: gh auth login"
    exit 1
fi

# Check if tag/release already exists
if gh release view "$VERSION" --repo "$GITHUB_REPO" &> /dev/null; then
    echo_warn "Release $VERSION already exists. Deleting and recreating..."
    gh release delete "$VERSION" --repo "$GITHUB_REPO" --yes
    # Also delete the tag if it exists
    git ls-remote --tags "https://github.com/$GITHUB_REPO.git" | grep -q "refs/tags/$VERSION" && \
        gh api -X DELETE "repos/$GITHUB_REPO/git/refs/tags/$VERSION" 2>/dev/null || true
fi

# Create the release with all artifacts
echo_info "Uploading release artifacts to GitHub..."
gh release create "$VERSION" \
    --repo "$GITHUB_REPO" \
    --title "Liminal v$VERSION" \
    --notes "Liminal v$VERSION" \
    "$OUTPUT_DIR/Liminal_${VERSION}_aarch64.dmg" \
    "$OUTPUT_DIR/Liminal_${VERSION}_x64.dmg" \
    "$OUTPUT_DIR/Liminal_${VERSION}_aarch64.app.tar.gz" \
    "$OUTPUT_DIR/Liminal_${VERSION}_aarch64.app.tar.gz.sig" \
    "$OUTPUT_DIR/Liminal_${VERSION}_x64.app.tar.gz" \
    "$OUTPUT_DIR/Liminal_${VERSION}_x64.app.tar.gz.sig" \
    "$OUTPUT_DIR/latest.json"

if [ $? -eq 0 ]; then
    echo_info "GitHub release created successfully!"
    echo_info "Release URL: https://github.com/$GITHUB_REPO/releases/tag/$VERSION"
else
    echo_error "Failed to create GitHub release"
    exit 1
fi

# Update latest.json in the repository root (for auto-update endpoint)
LATEST_JSON_PATH="$PROJECT_DIR/releases/latest.json"
cp "$OUTPUT_DIR/latest.json" "$LATEST_JSON_PATH"
echo_info "Updated $LATEST_JSON_PATH"

echo ""
echo_info "Release complete for version $VERSION!"
echo_info "Users can download from: https://github.com/$GITHUB_REPO/releases/tag/$VERSION"
