FROM rust:1.81.0-bookworm AS builder

WORKDIR /root/app
COPY --chown=root:root . .

RUN cargo build --release --bin babyrite

FROM debian:bookworm-slim AS runner

COPY --from=Builder --chown=root:root /root/app/target/release/babyrite /usr/local/bin/babyrite

RUN apt-get update \
    && apt-get install -y --no-install-recommends openssl \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --user-group babyrite
USER babyrite
WORKDIR /home/babyrite

LABEL org.opencontainers.image.source=https://github.com/m1sk9/babyrite

# https://github.com/m1sk9/babyrite/pull/208
ENV RUST_LOG=babyrite=info

ENTRYPOINT [ "sh", "-c", "babyrite" ]
