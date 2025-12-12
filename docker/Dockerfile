FROM rust:1.92.0-bookworm AS builder

WORKDIR /root/app
COPY --chown=root:root . .

RUN cargo build --release --bin babyrite

FROM gcr.io/distroless/cc-debian12 AS runner

COPY --from=builder --chown=root:root /root/app/target/release/babyrite /

LABEL org.opencontainers.image.source=https://github.com/m1sk9/babyrite

# https://github.com/m1sk9/babyrite/pull/208
ENV RUST_LOG=babyrite=info

CMD ["./babyrite"]
