#!/usr/bin/env bash

# This should be run from the root of the repository by someone with SSH access to dokku@builder.amethyst.rs


CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
TEMP_SCRIPT=".docs-helper.sh"

function die() {
  # Clean up after ourselves
  rm -rf ${TEMP_SCRIPT}
  git checkout "${CURRENT_BRANCH}"
}

# We need to make a copy of the script, in case we checkout a branch somewhere that
# doesn't contain it.
# Let's not do this when we move to CI if we can avoid it.
cp docs-helper.sh ${TEMP_SCRIPT} || die
chmod +x ${TEMP_SCRIPT} || die


