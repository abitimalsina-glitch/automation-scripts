#!/bin/bash

# projectchecker.sh
# Check the health of a software project.

set -u

PROJECT_NAME="${1:-}"

# -----------------------------
# Colors
# -----------------------------

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

PASS=0
WARN=0
FAIL=0

# -----------------------------
# Helper functions
# -----------------------------

pass() {
    echo -e "${GREEN}✓${NC} $1"
    ((PASS++))
}

warn() {
    echo -e "${YELLOW}⚠${NC} $1"
    ((WARN++))
}

fail() {
    echo -e "${RED}✗${NC} $1"
    ((FAIL++))
}

info() {
    echo -e "${BLUE}→${NC} $1"
}

# -----------------------------
# Check argument
# -----------------------------

if [ -z "$PROJECT_NAME" ]; then
    echo -e "${RED}Error:${NC} Please provide a project name."
    echo
    echo "Usage:"
    echo "  bash projectchecker.sh ByteCloud"
    exit 1
fi

# -----------------------------
# Find project
# -----------------------------

info "Searching for $PROJECT_NAME..."

PROJECT_DIR=$(find "$HOME" \
    -type d \
    -name "$PROJECT_NAME" \
    -not -path "*/node_modules/*" \
    -not -path "*/.git/*" \
    -print \
    -quit 2>/dev/null)

if [ -z "$PROJECT_DIR" ]; then
    echo -e "${RED}Error:${NC} Project '$PROJECT_NAME' was not found."
    exit 1
fi

# -----------------------------
# Move into project
# -----------------------------

PROJECT_DIR="$(cd "$PROJECT_DIR" && pwd)"

cd "$PROJECT_DIR" || exit 1

# -----------------------------
# Header
# -----------------------------

echo
echo "================================"
echo "        PROJECT CHECK"
echo "================================"
echo
echo "Project: $PROJECT_NAME"
echo "Path:    $PROJECT_DIR"
echo

# -----------------------------
# Git check
# -----------------------------

echo "----- Git -----"

if [ -d ".git" ]; then
    pass "Git repository found"

    if git diff --quiet && git diff --cached --quiet; then
        pass "No unstaged or staged changes"
    else
        warn "Uncommitted changes detected"

        echo
        git status --short
        echo
    fi

    if [ -n "$(git status --porcelain)" ]; then
        warn "Working tree is not clean"
    else
        pass "Working tree is clean"
    fi
else
    warn "Not a Git repository"
fi

echo

# -----------------------------
# Gitignore check
# -----------------------------

echo "----- Gitignore -----"

if [ -f ".gitignore" ]; then
    pass ".gitignore exists"
else
    warn ".gitignore is missing"
fi

echo

# -----------------------------
# bin / obj check
# -----------------------------

echo "----- Build Artifacts -----"

if [ -d "bin" ] || [ -d "obj" ]; then
    warn "bin/ or obj/ directory found"

    [ -d "bin" ] && info "bin/ exists"
    [ -d "obj" ] && info "obj/ exists"

    if [ -d ".git" ]; then
        TRACKED_BUILD_FILES=$(git ls-files | grep -E '(^|/)(bin|obj)/' || true)

        if [ -n "$TRACKED_BUILD_FILES" ]; then
            fail "Tracked files found inside bin/ or obj/"
            echo "$TRACKED_BUILD_FILES"
        else
            pass "bin/ and obj/ are not tracked by Git"
        fi
    fi
else
    pass "No bin/ or obj/ directories"
fi

echo

# -----------------------------
# Common junk check
# -----------------------------

echo "----- Common Junk -----"

JUNK_FOUND=0

if find . -name ".DS_Store" -type f -print -quit | grep -q .; then
    warn ".DS_Store files found"
    JUNK_FOUND=1
fi

if find . -name "__pycache__" -type d -print -quit | grep -q .; then
    warn "__pycache__ directories found"
    JUNK_FOUND=1
fi

if find . -name "*.pyc" -type f -print -quit | grep -q .; then
    warn ".pyc files found"
    JUNK_FOUND=1
fi

if [ "$JUNK_FOUND" -eq 0 ]; then
    pass "No common junk files found"
fi

echo

# -----------------------------
# Node.js checks
# -----------------------------

echo "----- Node.js -----"

if [ -f "package.json" ]; then
    pass "package.json found"

    if [ -d "node_modules" ]; then
        NODE_SIZE=$(du -sh node_modules 2>/dev/null | cut -f1)
        warn "node_modules exists ($NODE_SIZE)"
    else
        info "node_modules not found"
    fi

    if [ -f "pnpm-lock.yaml" ]; then
        pass "pnpm lockfile found"
    elif [ -f "package-lock.json" ]; then
        pass "npm lockfile found"
    elif [ -f "yarn.lock" ]; then
        pass "Yarn lockfile found"
    else
        warn "No dependency lockfile found"
    fi

    if [ -d ".next" ]; then
        NEXT_SIZE=$(du -sh .next 2>/dev/null | cut -f1)
        warn ".next build directory exists ($NEXT_SIZE)"
    fi
else
    info "Not a Node.js project"
fi

echo

# -----------------------------
# Large file check
# -----------------------------

echo "----- Large Files -----"

LARGE_FILES=$(find . \
    -type f \
    -not -path "./.git/*" \
    -size +100M \
    -print 2>/dev/null)

if [ -n "$LARGE_FILES" ]; then
    warn "Files larger than 100 MB found:"
    echo

    while IFS= read -r file; do
        SIZE=$(du -h "$file" 2>/dev/null | cut -f1)
        echo "  $SIZE  $file"
    done <<< "$LARGE_FILES"
else
    pass "No files larger than 100 MB"
fi

echo

# -----------------------------
# Disk space check
# -----------------------------

echo "----- Disk Space -----"

DISK_USAGE=$(df -h "$PROJECT_DIR" | awk 'NR==2 {print $5}' | tr -d '%')
DISK_FREE=$(df -h "$PROJECT_DIR" | awk 'NR==2 {print $4}')

info "Free disk space: $DISK_FREE"
info "Disk usage:      ${DISK_USAGE}%"

if [ "$DISK_USAGE" -ge 90 ]; then
    fail "Disk usage is above 90%"
elif [ "$DISK_USAGE" -ge 80 ]; then
    warn "Disk usage is above 80%"
else
    pass "Disk space looks healthy"
fi

echo

# -----------------------------
# Environment files
# -----------------------------

echo "----- Environment Files -----"

if find . -maxdepth 2 -name ".env" -type f -print -quit | grep -q .; then
    warn ".env file found"

    if [ -f ".gitignore" ] && grep -qxF ".env" .gitignore; then
        pass ".env is listed in .gitignore"
    else
        fail ".env may not be ignored by Git"
    fi
else
    pass "No .env file found"
fi

echo

# -----------------------------
# Summary
# -----------------------------

echo "================================"
echo "           SUMMARY"
echo "================================"
echo
echo -e "${GREEN}✓ Passed:${NC}   $PASS"
echo -e "${YELLOW}⚠ Warnings:${NC} $WARN"
echo -e "${RED}✗ Failed:${NC}   $FAIL"
echo

if [ "$FAIL" -gt 0 ]; then
    echo -e "${RED}Project needs attention.${NC}"
    exit 1
elif [ "$WARN" -gt 0 ]; then
    echo -e "${YELLOW}Project is usable, but has warnings.${NC}"
    exit 0
else
    echo -e "${GREEN}Project looks healthy.${NC}"
    exit 0
fi
