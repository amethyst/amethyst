#!/usr/bin/env bash

# This should be run by someone with SSH access to dokku@builder.amethyst.rs


CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)

function die() {
  # Clean up after ourselves
  rm -rf .gitlab-ci-helper.sh
  git checkout "${CURRENT_BRANCH}"
}

# We need to make a copy of the script, in case we checkout a branch somewhere that
# doesn't contain it.
# Let's not do this when we move to CI if we can avoid it.
cp gitlab-ci-helper.sh .gitlab-ci-helper.sh
chmod +x .gitlab-ci-helper.sh

