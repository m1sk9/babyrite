FROM rust:1.80.0-bookworm as Builder

WORKDIR /root/app
COPY --chown=root:root . .

RUN cargo build --release --bin babyrite

FROM debian:bookworm-slim as Runner

COPY --from=Builder --chown=root:root /root/app/target/release/babyrite /usr/local/bin/babyrite

RUN apt-get update \
    && apt-get install -y --no-install-recommends openssl \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --user-group babyrite
USER babyrite
WORKDIR /home/babyrite

LABEL org.opencontainers.image.source=https://github.com/m1sk9/babyrite

ENV RUST_LOG=babyrite=info

ENTRYPOINT [ "sh", "-c", "babyrite" ]
