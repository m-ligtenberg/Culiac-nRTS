#!/bin/bash
# Add to Changelog Script for Battle of Culiacán RTS
# Usage: ./add-to-changelog.sh <version> <change_type> <message>

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

print_success() {
    echo -e "${GREEN}✅${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}❌${NC} $1"
}

# Function to show usage
show_usage() {
    echo "Usage: $0 <version> <change_type> <message>"
    echo ""
    echo "Arguments:"
    echo "  version      Version number (e.g., '1.1.0')"
    echo "  change_type  One of: added, changed, deprecated, removed, fixed, security"
    echo "  message      Description of the change"
    echo ""
    echo "Examples:"
    echo "  $0 1.1.0 added 'New markdown to BlockDoc conversion feature'"
    echo "  $0 1.0.2 fixed 'Bug in HTML renderer causing incorrect output'"
    echo ""
    echo "The script will:"
    echo "  • Update CHANGELOG.md following Keep a Changelog format"
    echo "  • Create new version section if needed"
    echo "  • Add entry under appropriate change type"
    echo "  • Offer to commit the changes"
}

# Function to validate change type
validate_change_type() {
    local change_type="$1"
    case "$change_type" in
        added|changed|deprecated|removed|fixed|security)
            return 0
            ;;
        *)
            print_error "Invalid change type: $change_type"
            echo "Valid types: added, changed, deprecated, removed, fixed, security"
            return 1
            ;;
    esac
}

# Function to validate version format (basic semver check)
validate_version() {
    local version="$1"
    if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.-]+)?(\+[a-zA-Z0-9.-]+)?$ ]]; then
        print_error "Invalid version format: $version"
        echo "Version should follow semantic versioning (e.g., 1.0.0, 1.1.0-beta, 2.0.0+build.1)"
        return 1
    fi
}

# Function to get current date in YYYY-MM-DD format
get_current_date() {
    date +%Y-%m-%d
}

# Function to create changelog header if file doesn't exist
create_changelog_header() {
    cat > CHANGELOG.md << 'EOF'
# Changelog

All notable changes to Battle of Culiacán RTS will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

EOF
}

# Function to update Cargo.toml version if this is a new version
update_cargo_version() {
    local new_version="$1"
    local current_version
    
    current_version=$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
    
    if [[ "$current_version" != "$new_version" ]]; then
        print_info "Updating Cargo.toml version from $current_version to $new_version"
        sed -i "s/^version = \".*\"/version = \"$new_version\"/" Cargo.toml
        print_success "Updated Cargo.toml version to $new_version"
    fi
}

# Function to check if version section exists in changelog
version_exists_in_changelog() {
    local version="$1"
    grep -q "^## \[$version\]" CHANGELOG.md 2>/dev/null
}

# Function to add new version section
add_version_section() {
    local version="$1"
    local date="$2"
    local temp_file=$(mktemp)
    
    # Find the line with [Unreleased] and add new version after it
    awk -v version="$version" -v date="$date" '
        /^## \[Unreleased\]/ { 
            print $0
            print ""
            print "## [" version "] - " date
            print ""
            next
        }
        { print }
    ' CHANGELOG.md > "$temp_file"
    
    mv "$temp_file" CHANGELOG.md
    print_success "Added new version section [$version] - $date"
}

# Function to add entry to changelog
add_changelog_entry() {
    local version="$1"
    local change_type="$2"
    local message="$3"
    local temp_file=$(mktemp)
    local in_version_section=false
    local found_change_type=false
    local added_entry=false
    
    # Capitalize first letter of change type for display
    local display_type="$(tr '[:lower:]' '[:upper:]' <<< ${change_type:0:1})${change_type:1}"
    
    while IFS= read -r line; do
        # Check if we're entering the target version section
        if [[ "$line" =~ ^##\ \[$version\] ]]; then
            in_version_section=true
            echo "$line" >> "$temp_file"
            continue
        fi
        
        # Check if we're leaving the version section (next version or end)
        if [[ "$in_version_section" == true && "$line" =~ ^##\ \[ ]]; then
            # If we didn't find the change type section, add it before next version
            if [[ "$found_change_type" == false && "$added_entry" == false ]]; then
                echo "" >> "$temp_file"
                echo "### $display_type" >> "$temp_file"
                echo "- $message" >> "$temp_file"
                added_entry=true
            fi
            in_version_section=false
            found_change_type=false
        fi
        
        # If we're in the target version section and found the change type
        if [[ "$in_version_section" == true && "$line" =~ ^###\ $display_type ]]; then
            found_change_type=true
            echo "$line" >> "$temp_file"
            echo "- $message" >> "$temp_file"
            added_entry=true
            continue
        fi
        
        # If we're in the version section but hit another change type section
        if [[ "$in_version_section" == true && "$line" =~ ^###\  && "$found_change_type" == false ]]; then
            # Add our change type section before this one
            echo "" >> "$temp_file"
            echo "### $display_type" >> "$temp_file"
            echo "- $message" >> "$temp_file"
            echo "" >> "$temp_file"
            added_entry=true
            found_change_type=true
        fi
        
        echo "$line" >> "$temp_file"
    done < CHANGELOG.md
    
    # If we're still in the version section at EOF and haven't added the entry
    if [[ "$in_version_section" == true && "$added_entry" == false ]]; then
        echo "" >> "$temp_file"
        echo "### $display_type" >> "$temp_file"
        echo "- $message" >> "$temp_file"
    fi
    
    mv "$temp_file" CHANGELOG.md
    print_success "Added entry to $display_type section for version $version"
}

# Function to offer git commit
offer_commit() {
    local version="$1"
    local change_type="$2"
    local message="$3"
    
    echo ""
    print_info "Changes made to CHANGELOG.md and potentially Cargo.toml"
    
    read -p "Would you like to commit these changes? (y/N): " -n 1 -r
    echo
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        git add CHANGELOG.md Cargo.toml
        
        # Create commit message
        local commit_msg="update: Add changelog entry for v$version

$change_type: $message"
        
        git commit -m "$commit_msg"
        print_success "Changes committed successfully!"
    else
        print_info "Changes not committed. You can commit manually later."
    fi
}

# Main script logic
main() {
    # Check arguments
    if [[ $# -ne 3 ]]; then
        print_error "Wrong number of arguments"
        echo ""
        show_usage
        exit 1
    fi
    
    local version="$1"
    local change_type="$2"
    local message="$3"
    
    # Validate inputs
    validate_version "$version" || exit 1
    validate_change_type "$change_type" || exit 1
    
    # Check if we're in a git repository
    if [[ ! -d ".git" ]]; then
        print_warning "Not in a git repository. Continuing without git integration."
    fi
    
    # Create CHANGELOG.md if it doesn't exist
    if [[ ! -f "CHANGELOG.md" ]]; then
        print_info "CHANGELOG.md not found. Creating new changelog..."
        create_changelog_header
        print_success "Created CHANGELOG.md with standard header"
    fi
    
    # Check if version section exists
    if ! version_exists_in_changelog "$version"; then
        print_info "Version section [$version] not found. Creating new version section..."
        add_version_section "$version" "$(get_current_date)"
        
        # Update Cargo.toml version for new versions
        update_cargo_version "$version"
    fi
    
    # Add the changelog entry
    add_changelog_entry "$version" "$change_type" "$message"
    
    # Offer to commit if in git repo
    if [[ -d ".git" ]]; then
        offer_commit "$version" "$change_type" "$message"
    fi
    
    print_success "Changelog updated successfully!"
    echo ""
    print_info "Entry added: [$version] $change_type: $message"
}

# Run main function with all arguments
main "$@"