# List the just recipe list
list:
    just --list

# Mandatory checks to run before pushing changes to repository
checks:
    cargo fmt
    cargo build
    cargo clippy
    cargo test

# Clean artifacts and intermediate state
clean:
    find . -name target -type d -exec rm -r {} +
    just remove-lockfiles

# Remove lock files
remove-lockfiles:
    find . -name Cargo.lock -type f -exec rm {} +

# Test and build the book
book:
    mdbook build book
    mdbook test book -L ./target/debug/deps
