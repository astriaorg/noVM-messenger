default:
  @just --list

install-cli:
  cargo install --path ./rollup-cli --locked

compile-protos:
  cargo run --manifest-path tools/protobuf-compiler/Cargo.toml

test-rollup:
  @./test_rollup.sh

####################################################
## Scripts related to formatting code and linting ##
####################################################

default_lang := 'all'

# Can format 'rust', 'toml', 'proto', or 'all'. Defaults to all
fmt lang=default_lang:
  @just _fmt-{{lang}}

# Can lint 'rust', 'toml', 'proto', 'md' or 'all'. Defaults to all.
# Can also run the following sub-lints for rust: 'rust-fmt', 'rust-clippy',
# 'rust-clippy-custom', 'rust-clippy-tools', 'rust-dylint'
lint lang=default_lang:
  @just _lint-{{lang}}

_fmt-all:
  @just _fmt-rust
  @just _fmt-toml
  @just _fmt-proto

@_lint-all:
  -just _lint-rust
  -just _lint-toml
  -just _lint-proto
  -just _lint-md

[no-exit-message]
_fmt-rust:
  cargo +nightly-2024-09-15 fmt --all

[no-exit-message]
_lint-rust:
  just _lint-rust-fmt
  just _lint-rust-clippy
  just _lint-rust-clippy-custom
  just _lint-rust-clippy-tools
  just _lint-rust-dylint

[no-exit-message]
_lint-rust-fmt:
  cargo +nightly-2024-09-15 fmt --all -- --check

[no-exit-message]
_lint-rust-clippy:
  cargo clippy --all-targets --all-features \
          -- --warn clippy::pedantic --warn clippy::arithmetic-side-effects \
          --warn clippy::allow_attributes --warn clippy::allow_attributes_without_reason \
          --deny warnings

[no-exit-message]
_lint-rust-clippy-custom:
  cargo +nightly-2024-09-05 clippy --all-targets --all-features \
          -p tracing_debug_field \
          -- --warn clippy::pedantic --deny warnings

[no-exit-message]
_lint-rust-clippy-tools:
  cargo clippy --manifest-path tools/protobuf-compiler/Cargo.toml \
          --all-targets --all-features \
          -- --warn clippy::pedantic --deny warnings

[no-exit-message]
_lint-rust-dylint:
  cargo dylint --all --workspace

[no-exit-message]
_fmt-toml:
  taplo format

[no-exit-message]
_lint-toml:
  taplo format --check

[no-exit-message]
_lint-md:
  markdownlint-cli2

[no-exit-message]
_fmt-proto:
  buf format -w

[no-exit-message]
_lint-proto:
  buf lint
  buf format -d --exit-code
