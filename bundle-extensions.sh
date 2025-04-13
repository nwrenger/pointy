#!/bin/bash
set -e

EXT_DIR_ROOT="crates/extensions"
BUNDLE_OUTPUT_ROOT="bundled-extensions"
TARGET_PATH="target/release"

echo "📦 Bundling all extensions into $BUNDLE_OUTPUT_ROOT..."

case "$OSTYPE" in
  linux-gnu*)    EXT=".so";    OS_NAME="linux" ;;
  darwin*)       EXT=".dylib"; OS_NAME="mac" ;;
  msys*|cygwin*) EXT=".dll";   OS_NAME="windows" ;;
  *) echo "❌ Unsupported OS: $OSTYPE"; exit 1 ;;
esac

for dir in "$EXT_DIR_ROOT"/*/; do
  [ -d "$dir" ] || continue
  EXT_NAME=$(basename "$dir")
  echo "🔧 Bundling extension: $EXT_NAME"

  pushd "$dir" > /dev/null

  echo "🔨 Building in release mode..."
  cargo build --release

  # Adjust artifact path (workspace-aware)
  if [[ "$OS_NAME" == "windows" ]]; then
    ARTIFACT="../../../${TARGET_PATH}/${EXT_NAME}${EXT}"
  else
    ARTIFACT="../../../${TARGET_PATH}/lib${EXT_NAME}${EXT}"
  fi

  if [ ! -f "$ARTIFACT" ]; then
    echo "❌ Error: Build artifact not found: $ARTIFACT"
    popd > /dev/null
    continue
  fi

  OUTPUT_DIR="../../../$BUNDLE_OUTPUT_ROOT/$EXT_NAME"
  mkdir -p "$OUTPUT_DIR"

  DEST_FILE="${OS_NAME}${EXT}"
  echo "📁 Copying build output to $DEST_FILE..."
  cp "$ARTIFACT" "$OUTPUT_DIR/$DEST_FILE"

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
