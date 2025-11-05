FROM europe-west1-docker.pkg.dev/broxus-infrastructure/docker/rust-builder:stable AS builder

WORKDIR /build

# Build dependencies only, when source code changes,
# this build can be cached, we don't need to compile dependency again.
RUN mkdir src && touch src/lib.rs
COPY Cargo.toml Cargo.lock ./
RUN RUSTFLAGS=-g cargo build --release

# Build App
COPY . .
RUN touch src/lib.rs
RUN RUSTFLAGS=-g cargo build --release

FROM europe-west1-docker.pkg.dev/broxus-infrastructure/docker/rust-runtime:stable
COPY --from=builder /build/target/release/api /app/application
COPY --from=builder /build/entrypoint.sh /app/entrypoint.sh
USER runuser
EXPOSE 9000
ENTRYPOINT ["/app/entrypoint.sh"]
