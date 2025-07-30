# Changelog Management

This project includes an automated changelog management system that follows the [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format and [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Usage

Use the `/add-to-changelog` command to add entries to the project changelog:

```bash
./add-to-changelog.sh <version> <change_type> <message>
```

### Parameters

- **version**: Version number following semantic versioning (e.g., `1.1.0`, `2.0.0-beta`, `1.0.1+build.1`)
- **change_type**: One of the following:
  - `added` - New features
  - `changed` - Changes in existing functionality
  - `deprecated` - Soon-to-be removed features
  - `removed` - Removed features
  - `fixed` - Bug fixes
  - `security` - Security improvements
- **message**: Description of the change

### Examples

```bash
# Add a new feature
./add-to-changelog.sh 1.1.0 added "New markdown to BlockDoc conversion feature"

# Document a bug fix
./add-to-changelog.sh 1.0.2 fixed "Bug in HTML renderer causing incorrect output"

# Note a security improvement
./add-to-changelog.sh 1.0.3 security "Updated dependencies to fix CVE-2024-12345"

# Mark a feature as deprecated
./add-to-changelog.sh 2.0.0 deprecated "Legacy API endpoints will be removed in v3.0.0"
```

## What the Script Does

1. **Validates Input**: Ensures version follows semver and change type is valid
2. **Creates Version Section**: If the version doesn't exist, creates a new section with today's date
3. **Updates Cargo.toml**: Automatically updates the package version for new versions
4. **Adds Entry**: Places the entry under the appropriate change type section
5. **Offers Commit**: Prompts to commit changes to git (if in a git repository)

## Features

- ✅ **Semantic Versioning Validation**: Ensures version numbers follow semver format
- ✅ **Change Type Validation**: Only accepts valid Keep a Changelog change types
- ✅ **Auto Version Creation**: Creates new version sections with current date
- ✅ **Cargo.toml Integration**: Updates package version automatically
- ✅ **Git Integration**: Offers to commit changes with descriptive commit messages
- ✅ **Colorized Output**: Easy-to-read colored console output
- ✅ **Error Handling**: Comprehensive validation and error messages
- ✅ **Keep a Changelog Format**: Strictly follows the standard format

## File Structure

The script maintains the following changelog structure:

```markdown
# Changelog

All notable changes to Battle of Culiacán RTS will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.0] - 2025-07-30

### Added
- New feature description

### Fixed
- Bug fix description

## [1.0.0] - 2025-07-15

### Added
- Initial release features
```

## Error Handling

The script includes comprehensive error handling:

- **Invalid Version Format**: Validates semantic versioning format
- **Invalid Change Type**: Only accepts the six standard change types
- **Missing Arguments**: Shows usage help when arguments are missing
- **File Permissions**: Handles cases where files can't be written
- **Git Repository**: Gracefully handles non-git environments

## Integration with Development Workflow

The changelog management integrates seamlessly with the project's development workflow:

1. **Version Bumping**: Automatically updates `Cargo.toml` when adding new versions
2. **Git Commits**: Creates descriptive commit messages following project conventions
3. **Release Process**: Maintains accurate changelog for release notes
4. **Documentation**: Serves as project history documentation

## Development Notes

- The script is written in Bash for maximum compatibility
- Uses POSIX-compliant commands where possible
- Includes colored output for better UX
- Follows the project's existing code quality standards
- Designed to work with the existing git hook system