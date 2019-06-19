#! /bin/bash
#
# This script removes crate artifacts for which multiple built libraries exist.
#
# This is an attempt to balance the following:
#
# * Allow `mdbook test` to pass without hitting the `multiple matching crates` error.
# * Do not remove all built artifacts, so that previously built crates do not have to be rebuilt.
#
# If we do rely on multiple versions of a crate, then this will remove both versions, so there is
# value in maintaining consistency in all our dependency versions.

# Fail on error
set -e

profile="${profile:-debug}"

self_dir="$(dirname "$(readlink -f "${BASH_SOURCE}")")"
repository_dir="$(dirname "${self_dir}")"
target_dir="${repository_dir}/target"
target_deps_dir="${target_dir}/${profile}/deps"

# Early return
if ! test -d "${target_deps_dir}"; then return 0; fi

# This is safe because `-` in crate names are changed to `_`.
libs_with_duplicates="$(ls ${target_deps_dir} | grep -o 'lib[^-]\+' | sort | uniq)"

for lib in $libs_with_duplicates; do
  rm -rf "${target_deps_dir}/lib${lib}*"
done
