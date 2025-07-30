#!/bin/bash
# Git hooks setup script for Battle of Culiacán RTS
# Run this script to install development git hooks

set -e

echo "🔧 Setting up git hooks for Battle of Culiacán RTS..."

# Check if we're in a git repository
if [ ! -d ".git" ]; then
    echo "❌ Not in a git repository. Run this script from the project root."
    exit 1
fi

# Make sure hooks directory exists
mkdir -p .git/hooks

# Make hooks executable
chmod +x .git/hooks/pre-commit .git/hooks/commit-msg

echo "✅ Git hooks installed successfully!"
echo ""
echo "📋 Installed hooks:"
echo "  • pre-commit: Runs cargo fmt, cargo check, and cargo clippy"
echo "  • commit-msg: Validates commit message format"
echo ""
echo "💡 These hooks will help maintain code quality and consistent commit messages."
echo "🚀 You're ready to develop with automated quality checks!"