FROM rust:1.71-alpine as builder

RUN apk add --no-cache musl-dev
RUN mkdir /work
WORKDIR /work

COPY . .
RUN cargo build --release

FROM alpine
COPY --from=builder /work/target/release/ip-info /bin/ip-info
CMD [ "/bin/ip-info" ]
