
FROM rust:1.85.0-slim-bullseye AS build

RUN groupadd -g 1234 nrg && \
    useradd -m -u 1234 -g nrg nonroot

WORKDIR /build

RUN set -e
COPY . /build
RUN cargo build --release
RUN cp ./target/release/webeng /bin/server

FROM cgr.dev/chainguard/wolfi-base

COPY --from=build /bin/server /bin/

EXPOSE 8000

USER nonroot

CMD ["/bin/server"]
