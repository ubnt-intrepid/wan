FROM ubuntu:xenial

# Install dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      automake \
      autotools-dev \
      ca-certificates \
      g++ \
      gcc \
      git \
      libboost-dev \
      libboost-program-options-dev \
      libboost-system-dev \
      libcap-dev \
      libgcrypt11-dev \
      libicu-dev \
      libpcre3-dev \
      make \
      python \
      zlib1g-dev && \
    rm -rf /var/lib/apt/lists/*

# Download source files
RUN git clone --recursive https://github.com/melpon/wandbox.git /root/src/wandbox

# Install cattleshed
RUN cd /root/src/wandbox/cattleshed && \
    autoreconf -i && \
    ./configure \
      --prefix=/usr/local/cattleshed \
      --disable-install-setcap && \
    make && \
    make install

# Run cattleshed
ENV PATH $PATH:/usr/local/cattleshed/bin
ENV LD_LIBRARY_PATH $LD_LIBRARY_PATH:/usr/local/cattleshed/lib

CMD [ "cattleshed" ]
