#!/bin/bash

set -e

VERSION="${1:?Version number is missing}"
GIT_VERSION="v${VERSION}"

# Need Debian
export NAME=musikid EMAIL=musikid@outlook.com
dch -v "${VERSION}" "Initial release"
dch -r ""

sed -i "s/pkgver=.*/pkgver=${VERSION}/" PKGBUILD
sed -i '0,/version.*/s/version.*/version = "'"${VERSION}"'"/' **/Cargo.toml

cargo update

git cliff --tag "$VERSION" --output CHANGELOG.md

git add -A && git commit -m "chore(release): prepare for ${VERSION}"

export TEMPLATE="\
  {% for group, commits in commits | group_by(attribute=\"group\") %}
  {{ group | upper_first }}\
    {% for commit in commits %}
    - {{ commit.message | upper_first }} ({{ commit.id | truncate(length=7, end=\"\") }})\
      {% endfor %}
      {% endfor %}
"
changelog=$(git cliff --unreleased --strip all)
git tag -s -a "${GIT_VERSION}" -m "Release ${GIT_VERSION}" -m "${changelog}"
