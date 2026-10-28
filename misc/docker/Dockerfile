FROM docker.io/library/ubuntu:26.10@sha256:9c3a42b7c0016cbf87ceb71fd72d26c0e73b37e532e2c6071707f124c489c118

# curl is needed by Rust update tool
RUN apt-get update \
    && apt-get install -y curl build-essential \
        libheif-dev libheif-plugin-libde265 libheif-plugin-dav1d libheif-plugin-aomdec \
    && apt-get clean ; rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/* /usr/share/doc/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y # Download the latest stable Rust

ENV PATH="/root/.cargo/bin:${PATH}"

RUN cargo --version
