FROM rust:1.64 AS builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates

ENV USER=nopriv
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

WORKDIR /usr/app
COPY . /usr/app

RUN cargo build --target x86_64-unknown-linux-musl --release

FROM scratch

COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /usr/app

COPY --from=builder /usr/app/target/x86_64-unknown-linux-musl/release/ndsquared-rustapi ./
COPY Rocket.toml ./

USER nopriv:nopriv

ENTRYPOINT ["/usr/app/ndsquared-rustapi"]
