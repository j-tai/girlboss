[private]
help:
    @just --list

# Checks the project with all combinations of features.
check:
    just foreach cargo check

# Tests the project with all combinations of features.
test:
    just foreach cargo test --all-targets
    cargo test --doc --features tokio
    cargo test --doc --all-features

# Builds the documentation.
doc *args:
    RUSTDOCFLAGS='--cfg docsrs' cargo +nightly doc --all-features {{args}}

[private]
foreach *cmd:
    {{cmd}} --features tokio
    {{cmd}} --features actix-rt
    {{cmd}} --features tokio,actix-rt

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
    just check test
    git add Cargo.toml Cargo.lock
    git commit -m 'Bump version to {{version}}'
    cargo publish --all-features
    git tag 'v{{version}}' -m 'v{{version}}'
    git push
    git push --tags
