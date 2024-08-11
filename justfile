[private]
help:
    @just --list

# Publish the crate.
publish version:
    sed -i 's/^version = ".*/version = "{{version}}"/' Cargo.toml
    cargo test
    git add Cargo.toml
    git commit -m 'Bump version to {{version}}'
    cargo publish
    git tag 'v{{version}}' -m 'v{{version}}'
    git push --tags
