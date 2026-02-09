FROM rust:1.91.1-alpine AS builder
COPY . /app
WORKDIR /app
RUN apk add musl-dev
RUN cargo build --release

FROM alpine:3.22.1 AS deployer
RUN apk add --no-cache ca-certificates
RUN mkdir /app
WORKDIR /app
COPY --from=builder /app/target/release/bellboy /app/bellboy
ENTRYPOINT ["/app/bellboy"]
