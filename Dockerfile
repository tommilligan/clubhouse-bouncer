FROM frolvlad/alpine-rust as builder

# cache our dependencies
RUN USER=root cargo new --bin repo
WORKDIR /repo
COPY ./Cargo.lock ./Cargo.toml ./
RUN cargo build --release

# replace with project source and build
RUN rm -rf ./src
COPY ./src ./src
RUN cargo build --release

# runtime image
FROM alpine:3.8
# required to allow stacktrace/panic recovery
RUN apk add --no-cache libgcc

COPY --from=builder /repo/target/release/bouncer .
CMD ["./bouncer"]
