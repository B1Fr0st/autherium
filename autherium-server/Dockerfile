from rust:1.90.0-slim as build

RUN rustup target add x86_64-unknown-linux-musl && \
    apt-get update && \
    apt-get install -y --no-install-recommends musl-tools musl-dev && \
    update-ca-certificates 

RUN apt-get update && apt-get install -y adduser

COPY ./src ./src
COPY ./Cargo.lock .
COPY ./Cargo.toml .

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid 10001 \
    "autherium"


RUN cargo build --target x86_64-unknown-linux-musl --release


FROM rust:1.90-alpine3.20

COPY --from=build /etc/passwd /etc/passwd
COPY --from=build /etc/group /etc/group

USER autherium:autherium

COPY --from=build --chown=autherium:autherium ./target/x86_64-unknown-linux-musl/release/autherium-server /app/autherium-server

ENTRYPOINT ["./app/autherium-server"]