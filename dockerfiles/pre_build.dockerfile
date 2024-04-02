FROM debian:bookworm-slim AS runtime
ADD https://github.com/Feston229/PortMorph/releases/download/latest/port-morph_amd64.deb /tmp/port-morph.deb
RUN dpkg -i /tmp/port-morph.deb && rm /tmp/port-morph.deb
ENTRYPOINT ["/usr/bin/port_morph"]
