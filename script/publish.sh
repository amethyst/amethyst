#! /bin/bash
set -e

# The order is important because of inter-dependencies (see docs/PUBLISHING.md)
crates=(
  amethyst_config
  amethyst_derive
  amethyst_error
  amethyst_core
  amethyst_assets
  amethyst_network
  amethyst_window
  amethyst_audio
  amethyst_locale
  amethyst_input
  amethyst_controls
  amethyst_rendy
  amethyst_tiles
  amethyst_ui
  amethyst_utils
  amethyst_animation
  amethyst_gltf
  amethyst
  amethyst_test
)

for crate in "${crates[@]}"
do
  echo "Publishing ${crate}"

  if test "${crate}" = "amethyst"
  then
    cargo publish
  else
    (cd $crate && cargo publish)
  fi
  # Rate limit ourselves as `crates.io` takes a while to update cache.
  sleep 30
done