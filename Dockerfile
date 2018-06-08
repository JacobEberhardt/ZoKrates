FROM ubuntu:14.04

MAINTAINER JacobEberhardt <jacob.eberhardt@tu-berlin.de>, Dennis Kuhnert <dennis.kuhnert@campus.tu-berlin.de>

ARG RUST_TOOLCHAIN=nightly-2018-06-04
ARG LIBSNARK_PATH=/root/libsnark
ARG LIBSNARK_COMMIT=f7c87b88744ecfd008126d415494d9b34c4c1b20

WORKDIR /root/

RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    curl \
    libboost-all-dev \
    libgmp3-dev \
    libprocps3-dev \
    libssl-dev \
    pkg-config \
    python-markdown \
    git

ENV LD_LIBRARY_PATH $LD_LIBRARY_PATH:/usr/local/lib

RUN curl https://sh.rustup.rs -sSf | \
    sh -s -- --default-toolchain $RUST_TOOLCHAIN -y

ENV PATH=/root/.cargo/bin:$PATH

RUN git clone https://github.com/scipr-lab/libsnark.git $LIBSNARK_PATH
WORKDIR $LIBSNARK_PATH
RUN git checkout $LIBSNARK_COMMIT
RUN git submodule update --init --recursive

WORKDIR /root/

COPY . ZoKrates

RUN cd ZoKrates \
    && cargo build
