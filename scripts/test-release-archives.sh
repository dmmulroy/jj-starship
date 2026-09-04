#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
test_dir=$(mktemp -d)
trap 'rm -rf "$test_dir"' EXIT

mkdir -p \
  "$test_dir/artifacts/aarch64-apple-darwin" \
  "$test_dir/artifacts/x86_64-pc-windows-msvc"
printf 'unix binary' > "$test_dir/artifacts/aarch64-apple-darwin/jj-starship"
printf 'windows binary' > "$test_dir/artifacts/x86_64-pc-windows-msvc/jj-starship.exe"
chmod 644 "$test_dir/artifacts"/*/*

"$repo_root/scripts/create-release-archives.sh" \
  "$test_dir/artifacts" \
  "$test_dir/release"

unix_archive="$test_dir/release/jj-starship-aarch64-apple-darwin.tar.gz"
windows_archive="$test_dir/release/jj-starship-x86_64-pc-windows-msvc.zip"

test -f "$unix_archive"
test -f "$windows_archive"
test ! -e "$test_dir/release/jj-starship-x86_64-pc-windows-msvc.tar.gz"

tar_mode=$(tar -tvzf "$unix_archive" | awk '$NF == "jj-starship" { print substr($1, 1, 10) }')
test "$tar_mode" = '-rwxr-xr-x'
test "$(tar -xOzf "$unix_archive" jj-starship)" = 'unix binary'
test "$(unzip -p "$windows_archive" jj-starship.exe)" = 'windows binary'

printf 'release archive regression check passed\n'
