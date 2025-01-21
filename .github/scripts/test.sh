#!/bin/bash
set -e

export TERM=xterm-256color

# Statements waiting to be executed
statements=(
    "cargo clippy --no-default-features -- -D warnings"
    "cargo clippy -F gizmos -- -D warnings"
    "cargo clippy -F sprite -- -D warnings"
    "cargo clippy -F sprite,gizmos -- -D warnings"

    "cargo test --no-default-features"
    "cargo test -F gizmos"
    "cargo test -F sprite --doc"
    "cargo test --all-features"

    "cargo doc --no-deps --all-features"
)

# loop echo and executing statements
for statement in "${statements[@]}"; do
    echo "$(tput setaf 3)$statement$(tput sgr0)"
    eval $statement
    echo
done
