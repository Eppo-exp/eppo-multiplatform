#!/usr/bin/env sh
set -eux

VERSION="$(jq -r .version ./package.json)"

(cd native/sdk_core && cargo set-version -p sdk_core "$VERSION")

sed -e "s/\\(^ *version:\\).*/\1 \"$VERSION\",/" -i mix.exs
