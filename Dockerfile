FROM ekidd/rust-musl-builder as builder

COPY . .
RUN --mount=type=cache,target=/home/rust/.cargo/registry,uid=1000,gid=1000 --mount=type=cache,target=/home/rust/src/target,uid=1000,gid=1000 cargo install --path .

FROM alpine
COPY --from=builder /home/rust/.cargo/bin/ip-info /bin/ip-info
CMD [ "/bin/ip-info" ]
