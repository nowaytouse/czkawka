FROM docker.io/library/ubuntu:26.10@sha256:1c2a84ae3f2a24aeb97cded5b41f0211b441662f1a188cbd2e139e7625f6f1d6

# curl is needed by Rust update tool
RUN apt-get update \
    && apt-get install -y curl build-essential \
        libheif-dev libheif-plugin-libde265 libheif-plugin-dav1d libheif-plugin-aomdec \
    && apt-get clean ; rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/* /usr/share/doc/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain nightly

ENV PATH="/root/.cargo/bin:${PATH}"

RUN cargo --version
