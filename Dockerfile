FROM rust:1.50
MAINTAINER Alexis Le Provost <alexis.leprovost@outlook.com>

WORKDIR /usr/src/rust-backend-monitor
COPY Cargo.toml Cargo.lock diesel.toml scripts/run.sh ./
COPY src src
COPY migrations migrations

RUN cargo install diesel_cli --no-default-features --features mysql
RUN cargo install --path .
RUN setcap cap_net_raw=eip /usr/local/cargo/bin/rbm

ENTRYPOINT ["./run.sh"]
