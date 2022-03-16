#!/usr/bin/env bash

set -e

self_dir="$(realpath -- "$(dirname -- "$0")")"
project_dir="$(dirname -- "${self_dir}")"

verify_branch() {
    current_branch=$(git branch --show-current)
    if [ "${current_branch}" != "$1" ]; then
        >&2 echo "Error: current branch is not '${1}' but '${current_branch}'"
        return 1
    fi
    if [ -n "$(git status --porcelain)" ]; then
        >&2 echo "Error: there are uncommitted changes"
        return 1
    fi
}

cd "${project_dir}"

verify_branch "main"

printf "Running tests: "
if cargo test --quiet --release --all-features &> /dev/null; then
    echo "succeeded"
else
    echo "FAILED"
    >&2 echo "Error: test failed"
    exit 1
fi

cargo build --release

config_dir="${self_dir}/config"
recipe_dir="${self_dir}/recipes"
output_dir="${self_dir}/dist"

rm -rf "${config_dir}" "${output_dir}"

"${project_dir}/target/release/sweet-potator" \
    --config-dir "${config_dir}" \
    --recipe-dir "${recipe_dir}" \
    build "${output_dir}"

rm -rf "${config_dir}"

datetime=$(date -u +%Y-%m-%dT%TZ)
commit=$(git rev-parse --short HEAD)

git checkout "gh-pages"
verify_branch "gh-pages"

docs_dir="${project_dir}/docs"
rm -rf "${docs_dir}"

echo "move '${output_dir}' to '${docs_dir}'"
mv "${output_dir}" "${docs_dir}"

git add "${docs_dir}"
if [ -n "$(git status --porcelain)" ]; then
    git commit -m "Deploy main@${commit} (${datetime})"
else
    git status
fi

git checkout "main"
