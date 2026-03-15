#!/usr/bin/env bash
set -euo pipefail

# Produces a portable Linux bundle that includes the game binary plus runtime assets. (ref: DL-006)
# CI ships archives and smoke-tests the extracted app directory as the packaged-boot contract. (ref: DL-006)

workspace_root="${1:?workspace root required}"
dist_dir="${2:?dist dir required}"
artifact_name="${3:-game_app-linux-x86_64}"
binary_path="${workspace_root}/target/release/game_app"
staging_dir="${dist_dir}/${artifact_name}"
archive_path="${dist_dir}/${artifact_name}.tar.gz"

# The archive layout keeps a single top-level app directory so smoke scripts can locate the runnable package deterministically. (ref: DL-006)
# Portable archives keep the staged app directory self-contained with every runtime file the binary expects at boot. (ref: DL-006)
rm -rf "${staging_dir}" "${archive_path}"
mkdir -p "${staging_dir}"

cp "${binary_path}" "${staging_dir}/game_app"
cp -R "${workspace_root}/assets" "${staging_dir}/assets"

tar -C "${dist_dir}" -czf "${archive_path}" "${artifact_name}"
printf '%s\n' "${archive_path}"
