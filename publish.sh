#!/bin/bash

# Publish the book and the API documentation to the `gh-pages' branch.

# TODO: Only run if book or blog has been updated via git diff

cargo install mdbook
if [ $? -ne 0 ]; then
    exit
fi

cargo install --git https://github.com/cobalt-org/cobalt.rs
if [ $? -ne 0 ]; then
    exit
fi

./build_web.sh

sudo pip install ghp-import
ghp-import -n web/

git config user.name "Eyal Kalderon"
git config user.email "ebkalderon@gmail.com"
git push -fq https://${GH_TOKEN}@github.com/${TRAVIS_REPO_SLUG}.git gh-pages
