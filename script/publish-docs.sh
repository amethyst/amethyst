#!/usr/bin/env bash

# This script should be run from the root of the repository by someone with SSH access to dokku@builder.amethyst.rs

set -euxo pipefail

#--------------------
# VARS

CURRENT_BRANCH="$(git rev-parse --abbrev-ref HEAD)"
LATEST_RELEASE=$(git tag -l 'v*' | sort -V | tail -n 1)


if [[ $(uname) == "Darwin" ]] ; then
  BACKEND="metal"
else
  BACKEND="vulkan"
fi


# Cloudfront distribution ids
if [[ -z "$AWS_DOCS_DISTRIBUTION_ID" ]]; then
  die "AWS_DOCS_DISTRIBUTION_ID must be set"
fi
if [[ -z "$AWS_BOOK_DISTRIBUTION_ID" ]]; then
  die "AWS_BOOK_DISTRIBUTION_ID must be set"
fi

#--------------------
# FUNCTIONS

function cleanup() {
  echo "Cleaning up..."
  rm -f book-paths-updated
  rm -rf book-public
  rm -f docs-paths-updated
  rm -rf docs-public
  echo "Restoring branch: ${CURRENT_BRANCH}"
  git checkout -qf "${CURRENT_BRANCH}"
}

function die() {
  # Clean up after ourselves
  echo "Fatal error: $1"
  cleanup
  exit 2
}

function build_book {
  if [[ -z "$1" || -z "$2" || -z "$3" ]]; then
    die "Usage: build-book REF DIR INVALIDATION_PATH"
  fi

  REF=$1
  DIR=$2
  INVALIDATION_PATH=$3

  # Checkout the ref we were given
  git checkout -qf "$REF"
  HEAD_REV=$(git rev-parse HEAD)

  # Check if the existing ref matches
  if [[ -f "$DIR" && -f "${DIR}/.rev" && "$(cat ${DIR}/.rev)" = "$HEAD_REV" ]]; then
    die "Cached book build for $REF found in $DIR!"
  fi

  rm -rf "$DIR"
  mkdir -p "$DIR"
  echo "Building book for $REF..."
  # Prevent any book typos from "creating missing chapters" by disabling write access to book/src
  chmod -R a-w book/src
  mdbook build book -d "../$DIR"
  # Restore write access
  chmod -R ug+w book/src

  # Write the newly built rev to the rev file
  echo "$HEAD_REV" >> "$DIR"/.rev

  # Write the invalidation path since we just rebuilt
  echo "$INVALIDATION_PATH" >> ./book-paths-updated
}

function build_docs_wasm {
  if [[ -z "$1" || -z "$2" || -z "$3" ]]; then
    die "Usage: build-book REF DIR INVALIDATION_PATH"
  fi

  REF=$1
  DIR=$2
  INVALIDATION_PATH=$3

  # Checkout the ref we were given
  git checkout -qf "$REF"
  HEAD_REV=$(git rev-parse HEAD)

  # Check if the existing ref matches
  if [[ -f "$DIR" && -f "${DIR}/.rev" && "$(cat ${DIR}/.rev)" = "$HEAD_REV" ]]; then
    die "Cached docs build for $REF found in $DIR!"
  fi

  rm -rf "$DIR"
  mkdir -p "$DIR"
  echo "Building wasm docs for $REF..."

  # TODO: build wasm branch here
  # cargo doc --no-deps --workspace --features="animation gltf ${BACKEND}"

  # taken from run.sh for future reference:
  # # (cd amethyst_animation && cargo doc --no-deps)
  # (cd amethyst_assets && cargo doc --target wasm32-unknown-unknown --features="wasm" --no-deps)
  # (cd amethyst_audio && cargo doc --target wasm32-unknown-unknown --no-default-features --features="wasm vorbis wav" --no-deps)
  # (cd amethyst_config && cargo doc --target wasm32-unknown-unknown --no-deps)
  # (cd amethyst_controls && cargo doc --target wasm32-unknown-unknown --features="wasm" --no-deps)
  # (cd amethyst_core && cargo doc --target wasm32-unknown-unknown --no-default-features --features="wasm" --no-deps)
  # (cd amethyst_derive && cargo doc --target wasm32-unknown-unknown --no-deps)
  # (cd amethyst_error && cargo doc --target wasm32-unknown-unknown --no-deps)
  # # (cd amethyst_gltf && cargo doc --no-deps)
  # # (cd amethyst_input && cargo doc --target wasm32-unknown-unknown --features="wasm" --no-deps)
  # # (cd amethyst_locale && cargo doc --no-deps)
  # (cd amethyst_network && cargo doc --target wasm32-unknown-unknown --features="web_socket" --no-deps)
  # (cd amethyst_rendy && cargo doc --target wasm32-unknown-unknown --features="wasm gl" --no-deps)
  # # (cd amethyst_test && cargo doc --features="wasm gl web_socket" --no-deps)
  # # (cd amethyst_tiles && cargo doc --features="wasm gl web_socket" --no-deps)
  # # (cd amethyst_ui && cargo doc --target wasm32-unknown-unknown --no-default-features --features="gl wasm" --no-deps)
  # # (cd amethyst_utils && cargo doc --target wasm32-unknown-unknown --no-default-features --features="wasm" --no-deps)
  # (cd amethyst_window && cargo doc --target wasm32-unknown-unknown --no-default-features --features="wasm" --no-deps)
  # cargo doc --target wasm32-unknown-unknown --no-default-features --features="audio network renderer wasm vorbis wav gl web_socket" --no-deps
  # cd ..

  # mv target/doc/* $DIR/

  # Write the newly built rev to the rev file
  # echo "$HEAD_REV" >> $DIR/.rev

  # Write the invalidation path since we just rebuilt
  # echo "$INVALIDATION_PATH" >> ./docs-paths-updated
}

function build_docs {
  if [[ -z "$1" || -z "$2" || -z "$3" ]]; then
    die "Usage: build-book REF DIR INVALIDATION_PATH"
  fi

  REF=$1
  DIR=$2
  INVALIDATION_PATH=$3

  # Checkout the ref we were given
  git checkout -qf "$REF"
  HEAD_REV=$(git rev-parse HEAD)

  # Check if the existing ref matches
  if [[ -f "$DIR" && -f "${DIR}/.rev" && "$(cat ${DIR}/.rev)" = "$HEAD_REV" ]]; then
    echo "Cached docs build for $REF found in $DIR!"
    exit 0
  fi

  rm -rf "$DIR"
  mkdir -p "$DIR"
  echo "Building docs for $REF..."
  cargo doc --no-deps --workspace --features="animation gltf ${BACKEND}"
  mv target/doc/* "$DIR"/

  # Write the newly built rev to the rev file
  echo "$HEAD_REV" >> "$DIR"/.rev

  # Write the invalidation path since we just rebuilt
  echo "$INVALIDATION_PATH" >> ./docs-paths-updated
}


function invalidate_aws {
  # Check if there are any updated docs paths
  if [[ -f "docs-paths-updated" ]]; then
    echo "Invalidating cloudfront docs paths..."
    # Loop through updated docs paths and create an invalidation for each
    while read p; do
      echo "Creating invalidation for docs path $p"
      /usr/local/bin/aws cloudfront create-invalidation \
        --distribution-id "$AWS_DOCS_DISTRIBUTION_ID" \
        --paths "$p"
    done < docs-paths-updated
  fi

  # Check if there are any updated book paths
  if [[ -f "book-paths-updated" ]]; then
    echo "Invalidating cloudfront book paths..."
    # Loop through updated book paths and create an invalidation for each
    while read p; do
      echo "Creating invalidation for book path $p"
      /usr/local/bin/aws cloudfront create-invalidation \
        --distribution-id "$AWS_BOOK_DISTRIBUTION_ID" \
        --paths "$p"
    done < book-paths-updated
  fi
}

#-----------------
# ALL THE STUFF

# --- API DOCS ---

mkdir -p docs-public || die "Failed creating docs-public"
build_docs master docs-public/master "/master/*"
build_docs "${LATEST_RELEASE}" docs-public/stable "/stable/*"
#build_docs-wasm $BRANCH_WASM docs-public/wasm "/wasm/*"
touch docs-public/.static
cp docs-nginx.conf.sigil docs-public/app-nginx.conf.sigil

# Check if the index file contains the package name to validate the deployment
echo "/stable/amethyst/ amethyst" >> docs-public/CHECKS
echo "/master/amethyst/ amethyst" >> docs-public/CHECKS
# echo "/wasm/amethyst/ amethyst" >> docs-public/CHECKS

# Check if the revision file contains the revisions we just deployed
echo "/master/.rev $(cat docs-public/master/.rev)" >> docs-public/CHECKS
echo "/stable/.rev $(cat docs-public/stable/.rev)" >> docs-public/CHECKS
# echo "/wasm/.rev $(cat docs-public/wasm/.rev)" >> docs-public/CHECKS

echo "DEPLOYING DOCS, this may take a minute..."
tar c docs-public | ssh -o StrictHostKeyChecking=no dokku@builder.amethyst.rs tar:in docs-src

# --- BOOK ---

mkdir -p book-public || die "Failed creating book-public"
build_book master book-public/master "/master/*"
build_book "${LATEST_RELEASE}" book-public/stable "/stable/*"
#build_book $BRANCH_WASM book-public/wasm "/wasm/*"

touch book-public/.static
cp docs-nginx.conf.sigil book-public/app-nginx.conf.sigil

# Check if the index file contains the package name to validate the deployment
echo "/stable/ Amethyst" >> book-public/CHECKS
echo "/master/ Amethyst" >> book-public/CHECKS
# echo "/wasm/ Amethyst" >> book-public/CHECKS

# Check if the revision file contains the revisions we just deployed
echo "/master/.rev $(cat book-public/master/.rev)" >> book-public/CHECKS
echo "/stable/.rev $(cat book-public/stable/.rev)" >> book-public/CHECKS
# echo "/wasm/.rev $(cat book-public/wasm/.rev)" >> book-public/CHECKS

echo "DEPLOYING BOOK, this may take a minute..."
tar c book-public | ssh -o StrictHostKeyChecking=no dokku@builder.amethyst.rs tar:in book-src

# --- Invalidate AWS ---

invalidate_aws

# ---

cleanup
