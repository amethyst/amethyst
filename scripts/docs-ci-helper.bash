#!/bin/bash
set -euo pipefail

function build {
  if [[ -z "$1" || -z "$2" || -z "$3" || -z "$4" ]]; then
    echo "Usage: build (book|docs|docs-wasm) REF DIR INVALIDATION_PATH"
    exit 1
  fi

  local BUILD_TYPE="$1"
  if [[ $BUILD_TYPE != "docs" && $BUILD_TYPE != "book" && $BUILD_TYPE != "docs-wasm" ]]; then
    echo "Available build types: book, docs, docs-wasm"
    exit 1
  fi

  local REF="$2"
  local DIR="$3"
  local INVALIDATION_PATH="$4"

  # Checkout the ref we were given
  git checkout -q "$REF"
  HEAD_REV="$(git rev-parse HEAD)"

  # Check if the existing ref matches
  if [[ "$(cat "$DIR"/.rev)" = "$HEAD_REV" ]]; then
    echo "Cached $BUILD_TYPE build for $REF found in $DIR!"
    exit 0
  fi

  rm -rf "$DIR"
  mkdir -p "$DIR"
  echo "Building $BUILD_TYPE for $REF..."

  case $BUILD_TYPE in
    "docs")
      cargo doc --all --features="animation gltf vulkan"
      mv target/doc/* "$DIR/"
      ;;
    "docs-wasm")
      # TODO: build wasm branch here
      ;;
    "book")
      mdbook build book -d "../$DIR"
      ;;
    *)
      exit 1
  esac

  # Write the newly built rev to the rev file
  echo "$HEAD_REV" >> "$DIR/.rev"

  # Write the invalidation path since we just rebuilt
  if [[ $BUILD_TYPE = "docs" || $BUILD_TYPE = "docs-wasm" ]]; then
    echo "$INVALIDATION_PATH" >> ./docs-paths-updated
  elif [[ $BUILD_TYPE = "book" ]]; then
    echo "$INVALIDATION_PATH" >> ./book-paths-updated
  fi
}

function invalidate_aws {
  # Check if there are any updated docs paths
  if [[ -f "docs-paths-updated" ]]; then
    echo "Invalidating docs paths..."
    if [[ -z "$AWS_DOCS_DISTRIBUTION_ID" ]]; then
      echo "AWS_DOCS_DISTRIBUTION_ID must be set"
      exit 1
    fi

    # Loop through updated docs paths and create an invalidation for each
    while read -r p; do
      echo "Creating invalidation for docs path $p"
      /usr/local/bin/aws cloudfront create-invalidation \
        --distribution-id "$AWS_DOCS_DISTRIBUTION_ID" \
        --paths "$p"
    done < docs-paths-updated
  fi

  # Check if there are any updated book paths
  if [[ -f "book-paths-updated" ]]; then
    echo "Invalidating book paths..."
    if [[ -z "$AWS_BOOK_DISTRIBUTION_ID" ]]; then
      echo "AWS_BOOK_DISTRIBUTION_ID must be set"
      exit 1
    fi

    # Loop through updated book paths and create an invalidation for each
    while read -r p; do
      echo "Creating invalidation for book path $p"
      /usr/local/bin/aws cloudfront create-invalidation \
        --distribution-id "$AWS_BOOK_DISTRIBUTION_ID" \
        --paths "$p"
    done < book-paths-updated
  fi
}

SUBCOMMAND="$1"
case $SUBCOMMAND in
  "build")
    build "$2" "$3" "$4" "$5"
    ;;
  "invalidate-aws")
    invalidate_aws
    ;;
  *)
    echo "Usage: doc-ci-helper (build-book|build-docs|build-docs-wasm|invalidate-aws) [ARGS...]"
esac
