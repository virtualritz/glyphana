# Justfile for Glyphana

# Default recipe
default:
    @just --list

# Build the project in release mode
build:
    cargo build --release

# Run the application
run:
    cargo run

# Run tests
test:
    cargo test

# Format code
fmt:
    cargo fmt

# Run clippy
lint:
    cargo clippy --fix --allow-dirty

# Clean build artifacts
clean:
    cargo clean

# Package for the current platform using cargo-packager
package:
    #!/usr/bin/env sh
    set -e
    echo "Building packages using cargo-packager..."

    # Install cargo-packager if not installed
    if ! command -v cargo-packager &> /dev/null; then
        echo "Installing cargo-packager..."
        cargo install cargo-packager --locked
    fi

    # Run cargo-packager (it will detect the current platform automatically)
    cargo packager --release

    echo "Packages created successfully!"
    echo "Output location: dist/"

    # Show created packages
    if [ -d "dist" ]; then
        echo "Created packages:"
        ls -la dist/
    else
        echo "Warning: dist directory not found"
    fi

# Install packaging tools
install-packager:
    cargo install cargo-packager --locked

# Package with verbose output for debugging
package-verbose:
    cargo packager --release --verbose

# Package only specific formats (e.g., just package-deb, package-dmg, package-msi)
package-deb:
    cargo packager --release --formats deb

package-dmg:
    cargo packager --release --formats dmg

package-msi:
    cargo packager --release --formats msi

package-appimage:
    cargo packager --release --formats appimage

# Create source distribution
dist:
    @echo "Creating source distribution..."
    git archive --format=tar.gz --prefix=glyphana-$(shell cargo pkgid | cut -d# -f2)/ HEAD > glyphana-$(shell cargo pkgid | cut -d# -f2).tar.gz
    @echo "Source distribution created: glyphana-$(shell cargo pkgid | cut -d# -f2).tar.gz"