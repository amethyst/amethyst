#!bin/sh
if [ "$FORMATTING" = "true" ]; then
  # Check if cargo-update is present, if not install it.
  (cargo install-update --version || cargo install cargo-update) &&
  # Check if rustfmt is present, if not install it.
  if "rustfmt --version"; then
    cargo install-update -a;
  else
    cargo install rustfmt-nightly;
  fi &&
  cargo fmt -- --write-mode=diff;
else
  # Build and test without profiler
  cargo test --all -v &&
  # Build and test with profiler
  cargo test --all  --features profiler -v;
fi
