FROM rust:latest as builder

WORKDIR /app
COPY services .
RUN cargo build

FROM ubuntu:20.04 AS application

WORKDIR /app
COPY --from=builder /target/debug/server ./
ENTRYPOINT [ "/app/server" ]
