FROM rust:1.64

COPY . /usr/app
WORKDIR /usr/app

RUN cargo install --path .

ENTRYPOINT ["ndsquared-rustapi"]
