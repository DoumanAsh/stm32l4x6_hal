#!/bin/bash
if [ $TARGET != thumbv7em-none-eabihf ]; then
    echo "Skip since wrong target"
    exit 0
fi

if [ $TRAVIS_BRANCH != master ]; then
    echo "Skip since not master branch build"
    exit 0
fi

if [ $TRAVIS_PULL_REQUEST != false ]; then
    echo "Skip PR"
    exit 0
fi

REPO=`git config remote.origin.url`
SHA=`git log -1 --format="%s(%h %cd)" --date=short`
COMMIT_AUTHOR_EMAIL=`git --no-pager log -1 --pretty=format:"%ae" HEAD`

cargo install pulldown-cmark
touch index.html
echo '<div class="markdown-body">' >> index.html
cat README.md | pulldown-cmark >> index.html
echo '</div>' >> index.html
echo '<link rel="stylesheet" href="github-markdown.css"/>' >> index.html


git clone $REPO out/

cd out/

git checkout gh-pages || git checkout --orphan gh-pages

ls

rm -rf *
find . -name "_*.*" | xargs rm -rf
find . -name ".*.*" | xargs rm -rf

cp ../index.html .
cp ../ci/github-markdown.css .

mkdir STM32L496AG
mkdir STM32L476VG
cd ../
cargo doc --features STM32L476VG
cp -rf target/doc/* out/STM32L476VG
cargo doc --features STM32L496AG
cp -rf target/doc/* out/STM32L496AG

cd out/
git status

git config user.name "Travis CI"
git config user.email "$COMMIT_AUTHOR_EMAIL"
echo "https://${GIT_TOKEN}:x-oauth-basic@github.com\n" > ~/.git-credentials
git config remote.origin.url "https://${GIT_TOKEN}@github.com/${TRAVIS_REPO_SLUG}.git"

git add --all .
git commit -m "Auto-update" -m "Commit: ${SHA}"
git push origin HEAD
