FROM lukemathwalker/cargo-chef:latest-rust-1.93.1-bookworm AS chef
WORKDIR /root/app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS cook
COPY --from=planner /root/app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM cook AS builder
COPY . .
RUN cargo build --release --bin babyrite

FROM gcr.io/distroless/cc-debian12 AS runner

COPY --from=builder --chown=root:root /root/app/target/release/babyrite /

LABEL org.opencontainers.image.source=https://github.com/m1sk9/babyrite

# https://github.com/m1sk9/babyrite/pull/208
ENV RUST_LOG=babyrite=info

CMD ["./babyrite"]
