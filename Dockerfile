FROM rust:1.66-slim-buster as build-env

WORKDIR /build

# openssl-sys crate needs pkg-config
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY *.toml Cargo.* .
COPY src src
RUN cargo build --release

FROM gcr.io/distroless/cc-debian11

LABEL org.opencontainers.image.source=https://github.com/blenessy/reference-kbs \
    org.opencontainers.image.description="SEV Attestation Server" \
    org.opencontainers.image.licenses=Apache-2.0

COPY --from=build-env /build/Rocket.toml /build/target/release/reference-kbs /

CMD ["/reference-kbs"]
