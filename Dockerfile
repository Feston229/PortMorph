FROM alpine:latest

WORKDIR /ptm

COPY target/x86_64-unknown-linux-musl/debug/port_morph ./
COPY ptm.toml ./

RUN chmod +x ./port_morph

EXPOSE 9999

STOPSIGNAL SIGQUIT

CMD ["./target/x86_64-unknown-linux-musl/debug/port_morph"]
