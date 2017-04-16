#!/usr/bin/env sh
# pre-commit.sh

STASH_NAME="pre-commit-$(date +%s)"
RED='\033[1;31m'
GREEN='\033[1;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Check to make sure commit isn't emtpy
if git diff-index --quiet HEAD --; then
    echo "${RED} Empty commit"
else
    # Stash all changes in the working directory so we test only commit files
    if git stash save -u -k -q $STASH_NAME; then
        echo "${YELLOW}Stashed changes as:${NC} ${STASH_NAME}\n\n"
        stash=1
    fi
fi

cargo doc --no-deps &&
cargo build &&
# Build and test without profiler
cargo test --all &&
# Build and test with profiler
cargo test --all --features profiler

# Capture exit code from tests
status=$?

# Revert stash if changes were stashed to restor working directory files
if [ "$stash" -eq 1 ]
then
    if git stash pop -q; then
        echo "\n\n${GREEN}Reverted stash command${NC}"
    else
        echo "\n\n${RED}Unable to revert stash command${NC}"
    fi
fi

# Keep exit code from tests, so if they fail, prevent commit
exit $status
