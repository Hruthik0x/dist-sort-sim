#!/bin/bash

# Build the entire workspace
cargo build --workspace

# Run the distributor package with the provided arguments
cargo run -p distributor -- "$@"