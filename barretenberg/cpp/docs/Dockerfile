FROM alpine:3.18 AS builder
RUN apk update \
    && apk upgrade \
    && apk add --no-cache \
        doxygen
WORKDIR /usr/src/barretenberg/cpp/
COPY . .
RUN doxygen docs/Doxyfile
