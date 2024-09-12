FROM lukemathwalker/cargo-chef:latest AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
ARG features
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo install --path service --no-default-features --features $features

FROM debian:bookworm-slim as runner
COPY --from=builder /usr/local/cargo/bin/rust-client /usr/local/bin/rust-client
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates
RUN update-ca-certificates
EXPOSE 8080
CMD ["rust-client"]
