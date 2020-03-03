#! /bin/bash
set -e

# Release options
profile=debug

# Directories
self_dir="$(dirname "$(readlink -f "${BASH_SOURCE}")")"
repository_dir="$(dirname "${self_dir}")"
repository_manifest="${repository_dir}/Cargo.toml"
target_dir="${repository_dir}/target"
target_profile_dir="${target_dir}/${profile}"

coverage_dir="${target_dir}/coverage"
test -d "${coverage_dir}" || mkdir -p "${coverage_dir}"

kcov_exclude_line="kcov-ignore"
kcov_exclude_region="kcov-ignore-start:kcov-ignore-end"

if ! type jq 1>/dev/null 2>&1; then
  echo '`jq` is not installed. Please install it before continuing.'
  return 1
fi

# Builds all crates including tests, but don't run them yet.
# We will run the tests wrapped in `kcov`.
test_bins_by_crate="$(
    cargo test --all --no-run --features "empty" --message-format=json |
    jq -r "select(.profile.test == true) | (.package_id | split(\" \"))[0] + \";\" + .filenames[]"
  )"

# Set `LD_LIBRARY_PATH` so that tests can link against it
target_arch=$(rustup toolchain list | grep -F default | cut -d ' ' -f 1 | rev | cut -d '-' -f 1-4 | rev)
export LD_LIBRARY_PATH="${LD_LIBRARY_PATH}:$(rustc --print sysroot)/lib/rustlib/${target_arch}/lib/"

crate_coverage_dirs=()
for test_bin_by_crate in $test_bins_by_crate; do
  crate_name=${test_bin_by_crate%%;*}
  test_bin_path=${test_bin_by_crate##*;}
  test_bin_name=${test_bin_path##*/target/debug/}

  # Regular crates
  crate_dir="${repository_dir}/${crate_name}"

  # Special case workspace crate
  if grep -qF "name = \"${crate_name}\"" "${repository_manifest}" ; then
    crate_dir="${repository_dir}"
  fi

  test -d "${crate_dir}" || continue;

  crate_coverage_dir="${coverage_dir}/${test_bin_name}"
  crate_coverage_dirs+=("${crate_coverage_dir}")

  (
    echo "Running '${test_bin_path}'"

    export CARGO_MANIFEST_DIR="$crate_dir"
    kcov --include-pattern="${crate_dir}/src/,${crate_dir}/tests/" \
      "--exclude-line=${kcov_exclude_line}" \
      "--exclude-region=${kcov_exclude_region}" \
      "${crate_coverage_dir}" "${test_bin_path}"
  )
done

rm -rf "${coverage_dir}/merged"
kcov --merge "${coverage_dir}/merged" "${crate_coverage_dirs[@]}" \
  "--exclude-line=${kcov_exclude_line}" \
  "--exclude-region=${kcov_exclude_region}" \
