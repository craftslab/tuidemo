FROM rust:latest AS builder
WORKDIR /usr/src/tuidemo
COPY . .
RUN make install && \
    make build

FROM scratch
COPY --from=builder /usr/src/tuidemo/target/x86_64-unknown-linux-musl/release/tuidemo /usr/local/bin/tuidemo
ENTRYPOINT ["tuidemo"]
