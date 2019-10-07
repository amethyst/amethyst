# /bin/sh
#
# Clean amethyst build artifacts so `mdbook test` does not fail on multiple
# built libraries found.
#
# Allow nullglob so build doesn't fail if there were no build artifacts.

set -e

shopt -u | grep -q nullglob && changed=true && shopt -s nullglob

crates_to_clean=$(
  find book -type f -name '*.md' -exec grep -hF 'extern crate ' {} \+ |
  grep -o '[a-z0-9_]\+;$' |
  cut -d ';' -f 1 |
  sort |
  uniq
)

for crate in $crates_to_clean
do
  echo Running \""rm -rf target/debug/deps/lib${crate}-* target/debug/deps/${crate}-*"\"
  sh -c "rm -rf target/debug/deps/lib${crate}-* target/debug/deps/${crate}-*"
done

[ $changed ] && shopt -u nullglob; unset changed

exit 0
