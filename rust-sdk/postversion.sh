#!/usr/bin/env sh
set -euxo pipefail

VERSION="$(jq -r .version ./package.json)"

cargo set-version -p eppo "$VERSION"
