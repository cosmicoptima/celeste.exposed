FROM lukemathwalker/cargo-chef:latest-rust-1.56.0 AS chef
WORKDIR /app

FROM chef AS planner

COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 

RUN apt-get update
RUN apt-get install -y npm
RUN npm install -g browserify

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

RUN make

RUN cargo run --release