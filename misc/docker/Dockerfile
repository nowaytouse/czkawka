FROM docker.io/library/ubuntu:26.10@sha256:b5194b8e58989da0865ebc80078bf9f5f8742ec6a867c53f6ae699f6036756f7

# curl is needed by Rust update tool
RUN apt-get update \
    && apt-get install -y curl build-essential \
        libheif-dev libheif-plugin-libde265 libheif-plugin-dav1d libheif-plugin-aomdec \
    && apt-get clean ; rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/* /usr/share/doc/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain nightly

ENV PATH="/root/.cargo/bin:${PATH}"

RUN cargo --version
