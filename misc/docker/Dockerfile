FROM docker.io/library/ubuntu:26.10@sha256:262df65c0ec6bf8d9dcc87454f4330a05fae4b97825f45854af3b136666584c2

# curl is needed by Rust update tool
RUN apt-get update \
    && apt-get install -y curl build-essential \
        libheif-dev libheif-plugin-libde265 libheif-plugin-dav1d libheif-plugin-aomdec \
    && apt-get clean ; rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/* /usr/share/doc/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y # Download the latest stable Rust

ENV PATH="/root/.cargo/bin:${PATH}"

RUN cargo --version
