mod? argo 'dev/argo.just'
mod? helm 'dev/helm.just'

# Print this list
default:
  @just --list

# Build and install the rollup CLI
install-cli:
  cargo install --path ./crates/rollup-cli --locked

compile-protos:
  cargo run --manifest-path tools/protobuf-compiler/Cargo.toml

test-rollup:
  @./test_rollup.sh

default_docker_tag := 'local'
default_repo_name := 'ghcr.io/astriaorg'

# Builds docker image for the crate. Defaults to 'local' tag.
# NOTE: `_crate_short_name` is invoked as dependency of this command so that failure to pass a valid
# binary will produce a meaningful error message.
docker-build crate tag=default_docker_tag repo_name=default_repo_name: (_crate_short_name crate "quiet")
  #!/usr/bin/env sh
  set -eu
  short_name=$(just _crate_short_name {{crate}})
  set -x
  docker buildx build --load --build-arg TARGETBINARY={{crate}} -f containerfiles/Dockerfile -t {{repo_name}}/$short_name:{{tag}} .

# Maps a crate name to the shortened name used in the docker tag.
# If `quiet` is an empty string the shortened name will be echoed. If `quiet` is a non-empty string,
# the only output will be in the case of an error, where the input `crate` is not a valid one.
_crate_short_name crate quiet="":
  #!/usr/bin/env sh
  set -eu
  case {{crate}} in
    chat-rollup) short_name=chat-rollup ;;
    *) echo "{{crate}} is not a supported binary" && exit 2
  esac
  [ -z {{quiet}} ] && echo $short_name || true

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
