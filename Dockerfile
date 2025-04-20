
FROM rust:1.85.0-slim-bullseye AS build
ARG APP_NAME
WORKDIR /build

RUN set -e
COPY . /build
RUN cargo build --release
RUN cp ./target/release/$APP_NAME /bin/server

FROM cgr.dev/chainguard/wolfi-base

COPY --from=build /bin/server /bin/

USER nonroot

EXPOSE 8000

CMD ["/bin/server"]
