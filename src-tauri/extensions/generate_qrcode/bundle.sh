#!/bin/bash
set -e

############################################
# CONFIGURATION
############################################
EXTENSION_NAME="generate_qrcode"
OUTPUT_DIR="bundled"
ICON_NAME="icon.svg"


echo "Building extension ${EXTENSION_NAME} in release mode..."
cargo build --release


if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    ARTIFACT="target/release/lib${EXTENSION_NAME}.so"
    if [ ! -f "$ARTIFACT" ]; then
        echo "Error: Artifact $ARTIFACT not found!"
        exit 1
    fi
    cp "$ARTIFACT" dynamic_lib.so
    OS_ARTIFACT_NAME="linux.so"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    ARTIFACT="target/release/lib${EXTENSION_NAME}.dylib"
    if [ ! -f "$ARTIFACT" ]; then
        echo "Error: Artifact $ARTIFACT not found!"
        exit 1
    fi
    cp "$ARTIFACT" dynamic_lib.dylib
    OS_ARTIFACT_NAME="mac.dylib"
elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
    ARTIFACT="target/release/${EXTENSION_NAME}.dll"
    if [ ! -f "$ARTIFACT" ]; then
        echo "Error: Artifact $ARTIFACT not found!"
        exit 1
    fi
    cp "$ARTIFACT" dynamic_lib.dll
    OS_ARTIFACT_NAME="windows.dll"
else
    echo "Unsupported OS type: $OSTYPE"
    exit 1
fi


echo "Creating final extension folder structure..."
mkdir -p "$OUTPUT_DIR"

if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    mv dynamic_lib.so "$OUTPUT_DIR/${OS_ARTIFACT_NAME}"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    mv dynamic_lib.dylib "$OUTPUT_DIR/${OS_ARTIFACT_NAME}"
elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
    mv dynamic_lib.dll "$OUTPUT_DIR/${OS_ARTIFACT_NAME}"
fi

if [ -d assets ]; then
    cp assets/$ICON_NAME "$OUTPUT_DIR/icon.svg"
    cp assets/manifest.json "$OUTPUT_DIR/manifest.json"
else
    echo "Warning: assets directory not found. Skipping asset copy."
fi

echo "Extension packaged successfully in ${OUTPUT_DIR}."
