#!/bin/bash

# Build the entire workspace
cargo build --workspace

# Flags
has_custom_flag=false
ARGS=()

# First pass to detect custom flag presence
for arg in "$@"; do
  case "$arg" in
    --sasaki|--alternative|--odd-even)
      has_custom_flag=true
      break
      ;;
  esac
done

# Second pass to process and translate
while [[ $# -gt 0 ]]; do
  case "$1" in
    --sasaki)
      ARGS+=("-a" "2")
      shift
      ;;
    --alternative)
      ARGS+=("-a" "3")
      shift
      ;;
    --odd-even)
      ARGS+=("-a" "1")
      shift
      ;;
    -a|--algo)
      if $has_custom_flag; then
        shift 2  # skip both flag and value
      else
        ARGS+=("$1" "$2")
        shift 2
      fi
      ;;
    *)
      ARGS+=("$1")
      shift
      ;;
  esac
done

# Run the distributor package with the translated arguments
cargo run -p distributor -- "${ARGS[@]}"