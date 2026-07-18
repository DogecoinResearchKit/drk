FROM rust:1.93-bookworm AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates crates
# The server embeds the website at compile time via
# include_dir!("$CARGO_MANIFEST_DIR/website") in brk_website, where that path
# is a symlink to the top-level website/, which in turn symlinks
# scripts/modules -> ../../modules. Both real directories must be in the build
# context or the include_dir! step fails with a dangling symlink.
COPY website website
COPY modules modules

# Dogecoin requires the AuxPoW-aware decoder, selected via the `dogecoin` feature
# on brk_cli (the `brk` binary). A default (Bitcoin) build fails to parse
# merge-mined blocks from height 371,337 onward.
RUN cargo build --release --locked -p brk_cli --features dogecoin

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates procps && rm -rf /var/lib/apt/lists/*

RUN useradd -m -s /bin/bash brk

COPY --from=builder /app/target/release/brk /usr/local/bin/brk

USER brk

RUN mkdir -p /home/brk/.brk

EXPOSE 3110

WORKDIR /home/brk

ENTRYPOINT ["brk"]
