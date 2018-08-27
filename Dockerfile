# cached and compiled dependencies
FROM tommilligan/clubhouse-bouncer:builder as builder

# replace with project source and build
RUN rm -rf ./src
COPY ./src ./src
RUN cargo build --release

# runtime image
FROM alpine:3.8
# required to allow stacktrace/panic recovery
RUN apk add --no-cache libgcc libressl

COPY --from=builder /repo/target/release/bouncer .
ENV RUST_LOG=bouncer=info
CMD ["./bouncer"]

