FROM europe-west1-docker.pkg.dev/broxus-infrastructure/docker/rust-builder:stable AS builder

WORKDIR /build

# Build App
COPY . .
RUN RUSTFLAGS=-g cargo build --release

FROM europe-west1-docker.pkg.dev/broxus-infrastructure/docker/rust-runtime:stable
COPY --from=builder /build/openapi.yml /app/openapi.yml
COPY --from=builder /build/target/release/api /app/application
COPY --from=builder /build/entrypoint.sh /app/entrypoint.sh
USER runuser
EXPOSE 9000
ENTRYPOINT ["/app/entrypoint.sh"]
