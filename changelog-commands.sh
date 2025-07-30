#!/bin/bash
# Changelog Command Aliases for Battle of Culiacán RTS
# Source this file to enable /add-to-changelog commands
# Usage: source changelog-commands.sh

# Get the directory of this script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Function that implements the add-to-changelog command
add-to-changelog() {
    if [[ $# -ne 3 ]]; then
        echo "Usage: add-to-changelog <version> <change_type> <message>"
        echo ""
        echo "Example: add-to-changelog 1.1.0 added 'New feature implementation'"
        echo ""
        echo "Change types: added, changed, deprecated, removed, fixed, security"
        return 1
    fi
    
    # Execute the main script
    "$SCRIPT_DIR/add-to-changelog.sh" "$1" "$2" "$3"
}

# Note: Bash doesn't support aliases starting with '/'
# Use add-to-changelog instead of /add-to-changelog

# Export the function
export -f add-to-changelog

# Print success message when sourced
echo "✅ Changelog commands loaded successfully!"
echo ""
echo "Available commands:"
echo "  add-to-changelog <version> <change_type> <message>"
echo ""
echo "Example:"
echo "  add-to-changelog 1.1.0 added 'New markdown conversion feature'"
echo ""
echo "Note: For /add-to-changelog syntax, use the script directly:"
echo "  ./add-to-changelog.sh 1.1.0 added 'New feature'"