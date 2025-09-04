#!/bin/bash

# Git Feature Branch Creator for BMAD Method
# Usage: ./create-feature-branch.sh [story-name] [optional-branch-suffix]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Get current timestamp
TIMESTAMP=$(date +"%Y%m%d-%H%M%S")

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if git is installed
if ! command -v git &> /dev/null; then
    print_error "Git is not installed. Please install git first."
    exit 1
fi

# Check if we're in a git repository
if ! git rev-parse --is-inside-work-tree &> /dev/null; then
    print_error "Not in a git repository. Please run this from within your project."
    exit 1
fi

# Get story name from parameter or prompt
if [ -z "$1" ]; then
    print_warning "No story name provided."
    read -p "Enter story name (lowercase-with-dashes): " STORY_NAME
else
    STORY_NAME=$1
fi

# Clean story name (replace spaces with dashes, lowercase)
STORY_NAME=$(echo "$STORY_NAME" | tr '[:upper:]' '[:lower:]' | tr ' ' '-' | tr -cd '[:alnum:]-')

# Optional branch suffix
SUFFIX=${2:-""}
if [ -n "$SUFFIX" ]; then
    SUFFIX="-$SUFFIX"
fi

# Create branch name
BRANCH_NAME="feature/$STORY_NAME-$TIMESTAMP$SUFFIX"

# Get current branch
current_branch=$(git branch --show-current)
print_status "Current branch: $current_branch"

# Check if branch already exists
if git branch -a | grep -q "$BRANCH_NAME"; then
    print_warning "Branch $BRANCH_NAME already exists. Switching to it..."
    git checkout "$BRANCH_NAME"
else
    # Create and switch to new branch
    print_status "Creating new branch: $BRANCH_NAME"
    git checkout -b "$BRANCH_NAME"
    print_success "Created and switched to branch: $BRANCH_NAME"
fi

# Push branch to remote (if remote exists)
if git remote -v | grep -q origin; then
    print_status "Pushing branch to remote..."
    git push -u origin "$BRANCH_NAME"
    print_success "Branch pushed to origin/$BRANCH_NAME"
else
    print_warning "No remote 'origin' found. Branch created locally only."
fi

# Display branch information
print_success "Branch setup complete!"
echo
print_status "Branch details:"
echo "  Branch name: $BRANCH_NAME"
echo "  Created from: $current_branch"
echo "  Timestamp: $TIMESTAMP"
echo
print_status "Next steps:"
echo "  1. Make your changes"
echo "  2. Commit: git add . && git commit -m 'feat: your changes'"
echo "  3. Push: git push"
echo "  4. Create PR: gh pr create --title 'Feature: $STORY_NAME' --body 'Implements story $STORY_NAME'"

# Save branch info for later reference
echo "$BRANCH_NAME" > .current-feature-branch
echo "Created at: $(date)" >> .current-feature-branch
echo "Story: $STORY_NAME" >> .current-feature-branch

print_success "Branch info saved to .current-feature-branch"