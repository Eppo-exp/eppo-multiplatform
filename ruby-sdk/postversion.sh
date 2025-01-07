#!/usr/bin/env sh
set -eux

VERSION="$(jq -r .version ./package.json)"

cargo set-version -p eppo_client "$VERSION"

# Update VERSION in version.rb
sed -e "s/VERSION = \".*\"/VERSION = \"$VERSION\"/" -i ./lib/eppo_client/version.rb
