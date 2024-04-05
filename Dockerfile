FROM rust:1.77.1-bullseye as Builder

WORKDIR /root/app
COPY --chown=root:root . .

RUN cargo build --release --bin babyrite

FROM debian:bullseye-slim as Runner

COPY --from=Builder --chown=root:root /root/app/target/release/babyrite /usr/local/bin/babyrite

RUN useradd --create-home --user-group babyrite
USER babyrite
WORKDIR /home/babyrite

LABEL org.opencontainers.image.source=https://github.com/m1sk9/babyrite

ENTRYPOINT [ "sh", "-c", "RUST_LOG=babyrite=info babyrite" ]
