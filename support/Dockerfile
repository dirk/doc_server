FROM ubuntu:latest

ENV USER root
ENV RUST_VERSION=1.6.0

RUN apt-get update && \
  DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    curl \
    git \
    libssl-dev

RUN curl -s https://static.rust-lang.org/rustup.sh > /home/install.sh
RUN chmod +x /home/install.sh
RUN sh /home/install.sh --disable-sudo --yes

COPY script/build-doc.sh /home/build-doc.sh
RUN chmod +x /home/build-doc.sh

RUN mkdir -p /source
RUN chmod -R 777 /source
RUN adduser cargo --disabled-password --gecos ""
RUN usermod -u 1000 cargo
RUN usermod -G staff cargo
RUN chown -R cargo:staff /source

VOLUME /source
WORKDIR /source
