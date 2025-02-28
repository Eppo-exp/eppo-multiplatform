#!/usr/bin/env sh
set -eux

VERSION="$(jq -r .version ./package.json)"

(cd rust && cargo set-version -p eppo_sdk "$VERSION")

sed -e "s/^version:.*$/version: '$VERSION'/" -i ./pubspec.yaml
