[private]
help:
    @just --list

# Checks the project with all combinations of features.
check:
    cargo check --features tokio
    cargo check --features actix-rt
    cargo check --features tokio,actix-rt

# Analyze code coverage.
coverage:
    rm -rf target/package
    cargo tarpaulin --all-features --out=html --output-dir=target --skip-clean --target-dir=target/_tarpaulin
    #
    # Code coverage information has been written to target/tarpaulin-report.html
    #

# Publish the crate.
publish version:
    sed -i 's/^version = ".*/version = "{{version}}"/' Cargo.toml
    cargo test
    git add Cargo.toml Cargo.lock
    git commit -m 'Bump version to {{version}}'
    cargo publish
    git tag 'v{{version}}' -m 'v{{version}}'
    git push
    git push --tags
