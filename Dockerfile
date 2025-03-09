FROM rust:1.85.0-alpine
LABEL authors="extragornax"

COPY . /app
WORKDIR /app

RUN cargo build --release 

ENTRYPOINT ["/app/target/release/gpx_parse", "--webserver", "--webserver-bind", "0.0.0.0:8010"]