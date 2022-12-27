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

# NOTE: this needs to be changed and kept secret.
ENV WORKLOAD='{"workload_id":"sevtest","tee_config":"{\"flags\":{\"bits\":63},\"minfw\":{\"major\":0,\"minor\":0}}","passphrase":"mysecretpassphrase","launch_measurement":"3c6f91219614a28d2e193e82dc2366d1a758a52c04607999b5b8ff9216304c97"}'

CMD ["/reference-kbs"]
