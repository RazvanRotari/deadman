FROM rust:bookworm

COPY . /data
WORKDIR /data
ENV DATABASE_URL="sqlite://deadman.sqlite"
RUN cargo build --release 

FROM debian:bookworm 
LABEL version="1.0"

COPY --from=0 /data/target/release/deadman /workdir/deadman
RUN apt-get -y update && apt-get install -y openssl ca-certificates

HEALTHCHECK CMD curl 127.0.0.1:3000
WORKDIR /workdir
CMD /workdir/deadman
