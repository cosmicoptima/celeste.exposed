FROM rust:1.61.0

WORKDIR /usr/src/celeste
COPY . .

RUN cargo install --path .

CMD ["make", "&&", "cargo", "run", "--release"]