# List the recipe list
list:
    just --list

# Format entire code base
format:
    cargo fmt --all

# Build entire workspace
build:
    cargo build --all --all-features

# Test entire workspace
test:
    cargo test --all --all-features

# Run Clippy on entire workspace
clippy:
    cargo clippy --all --all-features

# Mandatory checks to run before pushing changes to repository
checks:
    just format
    just build
    just clippy
    just test

# Clean artifacts and intermediate state
clean:
    find . -name target -type d -exec rm -r {} +
    just remove-lockfiles

# Remove lock files
remove-lockfiles:
    find . -name Cargo.lock -type f -exec rm {} +

# List outdated dependencies
list-outdated:
    cargo outdated -R -w

# Update dependencies
update:
    cargo update --manifest-path ./crates/emergent/Cargo.toml --aggressive

# Build and test the book
book:
    mdbook build book
    mdbook test book -L ./target/debug/deps

# Publish workspace
publish:
    cargo publish --no-verify --manifest-path ./crates/emergent/Cargo.toml
