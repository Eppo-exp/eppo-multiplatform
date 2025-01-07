#!/usr/bin/env sh
set -eux

VERSION="$(jq -r .version ./package.json)"

cargo set-version -p eppo_py "$VERSION"
