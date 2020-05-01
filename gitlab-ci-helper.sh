#!/bin/bash
set -euxo pipefail

function build_book {
  if [[ -z "$1" || -z "$2" || -z "$3" ]]; then
    echo "Usage: build-book REF DIR INVALIDATION_PATH"
    exit 1
  fi

  REF=$1
  DIR=$2
  INVALIDATION_PATH=$3

  # Checkout the ref we were given
  git checkout -q $REF
  HEAD_REV=$(git rev-parse HEAD)

  # Check if the existing ref matches
  if [[ -f "$DIR" && -f "${DIR}/.rev" && "$(cat ${DIR}/.rev)" = "$HEAD_REV" ]]; then
    echo "Cached book build for $REF found in $DIR!"
    exit 0
  fi

  rm -rf $DIR
  mkdir -p $DIR
  echo "Building book for $REF..."
  mdbook build book -d "../$DIR"

  # Write the newly built rev to the rev file
  echo "$HEAD_REV" >> $DIR/.rev

  # Write the invalidation path since we just rebuilt
  echo "$INVALIDATION_PATH" >> ./book-paths-updated
}

function build_docs_wasm {
  if [[ -z "$1" || -z "$2" || -z "$3" ]]; then
    echo "Usage: build-book REF DIR INVALIDATION_PATH"
    exit 1
  fi

  REF=$1
  DIR=$2
  INVALIDATION_PATH=$3

  # Checkout the ref we were given
  git checkout -q $REF
  HEAD_REV=$(git rev-parse HEAD)

  # Check if the existing ref matches
  if [[ -f "$DIR" && -f "${DIR}/.rev" && "$(cat ${DIR}/.rev)" = "$HEAD_REV" ]]; then
    echo "Cached docs build for $REF found in $DIR!"
    exit 0
  fi

  rm -rf $DIR
  mkdir -p $DIR
  echo "Building wasm docs for $REF..."

  # TODO: build wasm branch here
  # cargo doc --all --features="animation gltf vulkan"

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
    echo "Usage: build-book REF DIR INVALIDATION_PATH"
    exit 1
  fi

  REF=$1
  DIR=$2
  INVALIDATION_PATH=$3

  # Checkout the ref we were given
  git checkout -q $REF
  HEAD_REV=$(git rev-parse HEAD)

  # Check if the existing ref matches
  if [[ -f "$DIR" && -f "${DIR}/.rev" && "$(cat ${DIR}/.rev)" = "$HEAD_REV" ]]; then
    echo "Cached docs build for $REF found in $DIR!"
    exit 0
  fi

  rm -rf $DIR
  mkdir -p $DIR
  echo "Building docs for $REF..."
  cargo doc --all --features="animation gltf vulkan"
  mv target/doc/* $DIR/

  # Write the newly built rev to the rev file
  echo "$HEAD_REV" >> $DIR/.rev

  # Write the invalidation path since we just rebuilt
  echo "$INVALIDATION_PATH" >> ./docs-paths-updated
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
    while read p; do
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
    while read p; do
      echo "Creating invalidation for book path $p"
      /usr/local/bin/aws cloudfront create-invalidation \
        --distribution-id "$AWS_BOOK_DISTRIBUTION_ID" \
        --paths "$p"
    done < book-paths-updated
  fi
}

SUBCOMMAND=$1
case $SUBCOMMAND in
  "build-docs")
    build_docs $2 $3 $4
    ;;
  "build-docs-wasm")
    build_docs_wasm $2 $3 $4
    ;;
  "build-book")
    build_book $2 $3 $4
    ;;
  "invalidate-aws")
    invalidate_aws
    ;;
  *)
    shift
    echo "Usage: gitlab-ci-helper (build-book|build-docs|build-docs-wasm|invalidate-aws) [ARGS...]"
esac