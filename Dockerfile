FROM rust:1.77-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app
COPY Cargo.toml ./
COPY src ./src

RUN cargo build --release

FROM alpine:3.19

RUN apk --no-cache add ca-certificates tzdata
WORKDIR /zenith

COPY --from=builder /app/target/release/zenith-gateway /zenith/zenith-gateway
COPY config/gateway.yaml /zenith/config/gateway.yaml

EXPOSE 8000 9090

ENTRYPOINT ["/zenith/zenith-gateway"]
