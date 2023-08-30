FROM rust:1.72.0-bullseye as Builder

WORKDIR /root/app
COPY --chown=root:root . .

RUN cargo build --release --bin babyrite --features enable_sentry

FROM debian:bullseye-slim as Runner

COPY --from=Builder --chown=root:root /root/app/target/release/babyrite /usr/local/bin/babyrite

RUN apt-get update && apt-get install -y libssl-dev ca-certificates

RUN useradd --create-home --user-group babyrite
USER babyrite
WORKDIR /home/babyrite

LABEL org.opencontainers.image.source=https://github.com/m1sk9/babyrite

ENTRYPOINT [ "babyrite" ]
