FROM rust:alpine AS build
RUN apk update && \
    apk add --no-cache musl-dev git openssl-dev
RUN git clone https://github.com/Feston229/PortMorph.git
WORKDIR /PortMorph
RUN cargo build --release

FROM alpine:edge AS runtime
WORKDIR /app
COPY --from=build /PortMorph/target/release/port_morph /usr/local/bin
ENTRYPOINT ["/usr/local/bin/port_morph"]