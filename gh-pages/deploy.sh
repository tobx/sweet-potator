#!/usr/bin/env bash

set -e

self_dir="$(realpath -- "$(dirname -- "$0")")"
project_dir="$(dirname -- "${self_dir}")"

cd "${project_dir}"

current_branch=$(git branch --show-current)

if [ "$current_branch" != "main" ]; then
    >&2 echo "Error: current branch is not 'main' but '${current_branch}'"
    exit 1
fi

if [ -n "$(git status --porcelain)" ]; then
    >&2 echo "Error: there are uncommitted changes"
    exit 1
fi

printf "Running tests: "
if cargo test --quiet --release --all-features &> /dev/null; then
    echo "succeeded"
else
    echo "FAILED"
    >&2 echo "Error: test failed"
    exit 1
fi

cargo build --release

config_dir="${self_dir}/sweet-potator"
recipe_dir="${self_dir}/recipes"
output_dir="${self_dir}/dist"

rm -rf "${config_dir}" "${output_dir}"

"${project_dir}/target/release/sweet-potator" \
    --config-dir "${config_dir}" \
    --recipe-dir "${recipe_dir}" \
    build "${output_dir}"

datetime=$(date -u +%Y-%m-%dT%TZ)
commit=$(git rev-parse --short HEAD)
git checkout "gh-pages"
git rm -rf .
echo "checkout '.gitignore'"
git checkout HEAD -- ".gitignore"
echo "copy from '${self_dir}/dist'"
cp -R "${self_dir}/dist/." .
git add .
if [ -n "$(git status --porcelain)" ]; then
    git commit -m "Deploy main@${commit} (${datetime})"
else
    git status
fi
git checkout "main"
