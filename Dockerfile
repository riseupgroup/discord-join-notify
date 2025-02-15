ARG RUST_VERSION=1.84.0
ARG APP_NAME=discord-join-notify

FROM rust:${RUST_VERSION} AS build
ARG APP_NAME
WORKDIR /app

RUN apt update && apt install -y musl-tools

RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    dpkgArch="$(dpkg --print-architecture)"; \
    case "${dpkgArch##*-}" in \
        i386) target="i686-unknown-linux-musl";; \
        amd64) target="x86_64-unknown-linux-musl";; \
        armhf) target="armv7-unknown-linux-musleabihf";; \
        arm64) target="aarch64-unknown-linux-musl";; \
        *) echo >&2 "unsupported architecture: ${dpkgArch}"; exit 1 ;; \
    esac; \
    rustup target add $target; \
    cargo build --release --target $target && \
    cp "./target/$target/release/$APP_NAME" /bin/$APP_NAME

FROM alpine:3.21 AS final
ARG APP_NAME
WORKDIR /config

COPY --from=build /bin/$APP_NAME /bin/$APP_NAME

CMD ["discord-join-notify"]
