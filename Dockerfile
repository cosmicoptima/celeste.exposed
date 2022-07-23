FROM rust:1.61.0

WORKDIR /usr/src/celeste
COPY . .

RUN make

CMD ["cargo", "run", "--release"]