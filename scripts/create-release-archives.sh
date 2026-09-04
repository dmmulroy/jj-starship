#!/usr/bin/env bash
set -euo pipefail

artifacts_dir=${1:-artifacts}
release_dir=${2:-release}

mkdir -p "$release_dir"
artifacts_dir=$(cd "$artifacts_dir" && pwd)
release_dir=$(cd "$release_dir" && pwd)

for platform_dir in "$artifacts_dir"/*/; do
  target=$(basename "$platform_dir")

  case "$target" in
    *-pc-windows-*)
      (cd "$platform_dir" && zip -q "$release_dir/jj-starship-${target}.zip" jj-starship.exe)
      ;;
    *)
      # actions/upload-artifact does not preserve file permissions.
      chmod 755 "$platform_dir/jj-starship"
      (cd "$platform_dir" && tar -czf "$release_dir/jj-starship-${target}.tar.gz" jj-starship)
      ;;
  esac
done
