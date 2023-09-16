FROM rust:1-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /usr/src/app

COPY . .

ENV SQLX_OFFLINE=true

RUN cargo install --path .

FROM alpine:latest

WORKDIR /app

COPY --from=builder /usr/local/cargo/bin/the-martian-bot /app/the-martian-bot

CMD ["/app/the-martian-bot"]