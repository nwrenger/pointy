#!/bin/bash
set -e

EXT_DIR_ROOT="crates/extensions"
BUNDLE_OUTPUT_ROOT="bundled-extensions"

# If running on an ARM64 host (e.g. Apple M1/M2), set Docker's default platform to linux/amd64.
if [[ "$(uname -m)" == "arm64" || "$(uname -m)" == "aarch64" ]]; then
    echo "Detected ARM64 host; setting DOCKER_DEFAULT_PLATFORM=linux/amd64"
    export DOCKER_DEFAULT_PLATFORM=linux/amd64
fi

# Define the OS parameters.
OS_NAMES=("linux" "mac" "windows")
TARGET_TRIPLES=("x86_64-unknown-linux-gnu" "x86_64-apple-darwin" "x86_64-pc-windows-gnu")
LIB_EXTENSIONS=(".so" ".dylib" ".dll")

echo "📦 Bundling all extensions into $BUNDLE_OUTPUT_ROOT..."

# Loop through each extension directory.
for dir in "$EXT_DIR_ROOT"/*/; do
  [ -d "$dir" ] || continue
  EXT_NAME=$(basename "$dir")
  echo "🔧 Bundling extension: $EXT_NAME"

  pushd "$dir" > /dev/null

  # Create the output directory for this extension.
  OUTPUT_DIR="../../../$BUNDLE_OUTPUT_ROOT/$EXT_NAME"
  mkdir -p "$OUTPUT_DIR"

  # Build for every defined target.
  for i in "${!OS_NAMES[@]}"; do
      OS_NAME="${OS_NAMES[$i]}"
      TARGET_TRIPLE="${TARGET_TRIPLES[$i]}"
      EXT="${LIB_EXTENSIONS[$i]}"

      echo "🔨 Building for $OS_NAME ($TARGET_TRIPLE) in release mode..."
      cross build --target "$TARGET_TRIPLE" --release

      # Determine the expected artifact location.
      # Rust prepends "lib" for non-Windows targets.
      if [[ "$OS_NAME" == "windows" ]]; then
        ARTIFACT="../../../target/$TARGET_TRIPLE/release/${EXT_NAME}${EXT}"
      else
        ARTIFACT="../../../target/$TARGET_TRIPLE/release/lib${EXT_NAME}${EXT}"
      fi

      if [ ! -f "$ARTIFACT" ]; then
        echo "❌ Error: Build artifact not found: $ARTIFACT"
        continue
      fi

      DEST_FILE="${OS_NAME}${EXT}"
      echo "📁 Copying build output to $DEST_FILE..."
      cp "$ARTIFACT" "$OUTPUT_DIR/$DEST_FILE"
  done

  # Copy assets if present.
  if [ -d "assets" ]; then
    echo "🖼️ Copying assets..."
    cp "assets/icon.svg" "$OUTPUT_DIR/icon.svg" 2>/dev/null || echo "⚠️ Icon not found, skipping"
    cp "assets/manifest.json" "$OUTPUT_DIR/manifest.json" 2>/dev/null || echo "⚠️ Manifest not found, skipping"
  else
    echo "⚠️ No assets folder found in $EXT_NAME"
  fi

  popd > /dev/null
  echo "✅ $EXT_NAME bundled to $OUTPUT_DIR"
  echo
done

echo "🎉 All extensions bundled to: $BUNDLE_OUTPUT_ROOT/"
