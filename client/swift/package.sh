#!/bin/sh
set -x

BUILD_PROFILE="debug"
FRAMEWORK_NAME="OrderbookClient"
FRAMEWORK_FILENAME=$FRAMEWORK_NAME
WORKING_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
MANIFEST_PATH="$WORKING_DIR/../Cargo.toml"
CRATE_NAME="$(grep --max-count=1 '^name =' "$MANIFEST_PATH" | cut -d '"' -f 2)"
LIB_NAME="liborderbook_client.a"
TARGET_DIR="$WORKING_DIR/../../target"
XCFRAMEWORK_ROOT="$WORKING_DIR/$FRAMEWORK_FILENAME.xcframework"

rm -rf "$XCFRAMEWORK_ROOT"

# Build the directory structure right for an individual framework.
# Most of this doesn't change between architectures.

COMMON="$XCFRAMEWORK_ROOT/common/$FRAMEWORK_NAME.framework"

mkdir -p "$COMMON/Modules"
#cp "$TARGET_DIR/${FRAMEWORK_NAME}.modulemap" "${COMMON}/Modules/module.modulemap"
cp "module.modulemap" "${COMMON}/Modules/"

mkdir -p "$COMMON/Headers"
cp "$TARGET_DIR/${FRAMEWORK_NAME}.h" "$COMMON/Headers/"

cp "$TARGET_DIR/${FRAMEWORK_NAME}.swift" "./"

# iOS hardware
mkdir -p "$XCFRAMEWORK_ROOT/ios-arm64"
cp -r "$COMMON" "$XCFRAMEWORK_ROOT/ios-arm64/$FRAMEWORK_NAME.framework"
cp "$TARGET_DIR/aarch64-apple-ios/$BUILD_PROFILE/$LIB_NAME" "$XCFRAMEWORK_ROOT/ios-arm64/$FRAMEWORK_NAME.framework/$FRAMEWORK_NAME"

# iOS simulator, with both platforms as a fat binary for mysterious reasons
mkdir -p "$XCFRAMEWORK_ROOT/ios-arm64_x86_64-simulator"
cp -r "$COMMON" "$XCFRAMEWORK_ROOT/ios-arm64_x86_64-simulator/$FRAMEWORK_NAME.framework"
lipo -create \
  -output "$XCFRAMEWORK_ROOT/ios-arm64_x86_64-simulator/$FRAMEWORK_NAME.framework/$FRAMEWORK_NAME" \
  "$TARGET_DIR/aarch64-apple-ios-sim/$BUILD_PROFILE/$LIB_NAME" \
  "$TARGET_DIR/x86_64-apple-ios/$BUILD_PROFILE/$LIB_NAME"

# Set up the metadata for the XCFramework as a whole.
cp "$WORKING_DIR/Info.plist" "$XCFRAMEWORK_ROOT/Info.plist"

rm -rf "$XCFRAMEWORK_ROOT/common"
