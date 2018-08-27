# this is to build cargo packages natively for alpine
FROM tommilligan/clubhouse-bouncer:base

RUN apk add --no-cache libressl libressl-dev

# cache our dependencies
RUN USER=root cargo new --bin repo
WORKDIR /repo
COPY ./Cargo.lock ./Cargo.toml ./
RUN cargo build --release

