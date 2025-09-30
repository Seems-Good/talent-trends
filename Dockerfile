# Build stage with Rust official image
FROM rust:alpine AS builder

RUN apk add pkgconf openssl-dev openssl-libs-static build-base musl-utils

WORKDIR /app

# Copy source code and Cargo files
COPY . .

# Build release binary
RUN cargo build --release

# Runtime stage using minimal alpine
FROM alpine:latest

RUN apk add --no-cache libc6-compat

WORKDIR /app

COPY --from=builder /app/target/release/talent-trends .

EXPOSE 3000

CMD ["./talent-trends"]

