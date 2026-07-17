#!/usr/bin/env bash
set -e

# Get the directory of the script
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
# Assuming the script is in scripts/, go up one level to project root
PROJECT_ROOT="$SCRIPT_DIR/.."

SRC_README="$PROJECT_ROOT/README.md"
DEST_README="$PROJECT_ROOT/docs/README.md"

echo "Syncing $SRC_README to $DEST_README..."

# Read content, replace 'docs/' with empty string for links, and write to destination
cat "$SRC_README" | sed 's|(docs/|(|g' > "$DEST_README"

echo "Sync complete."
echo "Building mdBook..."

# Run mdbook build from project root
cd "$PROJECT_ROOT"

if command -v mdbook &> /dev/null; then
    echo "Building mdBook..."
    mdbook build
    echo "Build successful! Output is in 'book/' directory."
else
    echo "mdbook not found in PATH. Skipping documentation build."
fi
