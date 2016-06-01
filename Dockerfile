FROM debian:jessie
MAINTAINER Daniel Milholland <Milhound@icloud.com>

ENV USER root
ENV DEBIAN_FRONTEND noninteractive
ENV RUST_VERSION=1.9.0

# Install basic dependencies
RUN echo 'deb http://www.deb-multimedia.org jessie main non-free' >> /etc/apt/sources.list \
    && echo 'deb-src http://www.deb-multimedia.org jessie main non-free' >> /etc/apt/sources.list \
    && apt-get update \
        --quiet \
    && apt-get install \
            -y \
            --force-yes \
            --no-install-recommends \
            --no-install-suggests \
        deb-multimedia-keyring \
        build-essential \
        ca-certificates \
        curl \
        wget \
        git \
        libssl-dev \
        youtube-dl \
        libopus0 \
        ffmpeg

# Create Direcotry for binaries
RUN mkdir software \
    && cd software

# Install libsodium
RUN curl https://download.libsodium.org/libsodium/releases/libsodium-1.0.10.tar.gz | tar xz \
    && cd libsodium-1.0.10 \
    && ./configure \
    && make \
    && make check \
    && make install \
    && cd ..

# Install Rust-lang
RUN curl -sO https://static.rust-lang.org/dist/rust-$RUST_VERSION-x86_64-unknown-linux-gnu.tar.gz \
    && tar -xzf rust-$RUST_VERSION-x86_64-unknown-linux-gnu.tar.gz \
    && ./rust-$RUST_VERSION-x86_64-unknown-linux-gnu/install.sh --without=rust-docs

# Clean Up
RUN apt-get remove --purge -y curl \
    && apt-get remove --purge -y wget \
    && apt-get autoremove -y \
    && apt-get clean \
    && rm -rf \
    rust-$RUST_VERSION-x86_64-unknown-linux-gnu \
    rust-$RUST_VERSION-x86_64-unknown-linux-gnu.tar.gz \
    /var/lib/apt/lists/* \
    /tmp/* \
    /var/tmp/* \
    && mkdir /source

VOLUME ["/source"]
WORKDIR /source
CMD ["cargo","update"]
CMD ["cargo", "run"]
