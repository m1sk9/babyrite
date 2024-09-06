#!/bin/bash

COMMAND=$1

MDBOOK_VERSION='0.4.40'
MDBOOK_PAGETOC_VERSION='0.2.0'
MDBOOK_ALERTS_VERSION='0.5.2'

case $COMMAND in
  build)
    echo "Building ..."
    cargo build
    echo "Done! Run with ./target/debug/$(basename $PWD)"
    ;;
  build:docs)
    echo "Building (babyrite docs) ..."
    (cd docs && mdbook build)
    echo "Done!"
    ;;
  build:release)
    echo "Building ... (release)"
    cargo build --release
    (cd docs && mdbook build)
    echo "Done! Run with ./target/release/$(basename $PWD)"
    ;;
  init)
    cp .env.example .env
    echo "Initialized .env"
    ;;
  docs-init)
    echo "Installing mdbook (and plugins) ..."
    cargo install mdbook --vers $MDBOOK_VERSION
    cargo install mdbook-pagetoc --vers $MDBOOK_PAGETOC_VERSION
    cargo install mdbook-alerts --vers $MDBOOK_ALERTS_VERSION
    echo "Done!"
    ;;
  check)
    cargo check
    ;;
  check:docs)
    echo "Checking (babyrite docs) ..."
    (cd docs && mdbook check)
    echo "Done!"
    ;;
  doc)
    cargo doc --open
    ;;
  dev)
    echo "Running ..."
    cargo run &
    (cd docs && mdbook serve) &
    ;;
  clean)
    echo "Cleaning ..."
    cargo clean
    (cd docs && mdbook clean)
    echo "Done!"
    ;;
  *)
    echo "Usage: x {build|build:docs|build:release|init|docs-init|check|check:docs|doc|dev|clean}"
    ;;
esac
