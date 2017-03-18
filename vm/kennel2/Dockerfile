FROM ubuntu:xenial

# Install dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      automake \
      autotools-dev \
      ca-certificates \
      cmake \
      g++ \
      gcc \
      git \
      libcap-dev \
      libgcrypt11-dev \
      libicu-dev \
      libpcre3-dev \
      libsqlite3-dev \
      make \
      python \
      rsync \
      zlib1g-dev && \
    rm -rf /var/lib/apt/lists/*

# Download source files
RUN git clone --recursive https://github.com/melpon/cppcms.git /root/src/cppcms
RUN git clone --recursive https://github.com/melpon/cppdb.git /root/src/cppdb
RUN git clone --recursive https://github.com/melpon/wandbox.git /root/src/wandbox

# Install CppCMS
RUN mkdir -p /root/build/cppcms && \
    cd /root/build/cppcms && \
    cmake /root/src/cppcms/ \
      -DCMAKE_BUILD_TYPE=Release \
      -DCMAKE_INSTALL_PREFIX=/usr/local/cppcms \
      -DDISABLE_SHARED=ON \
      -DDISABLE_FCGI=ON \
      -DDISABLE_SCGI=ON \
      -DDISABLE_ICU_LOCALE=ON \
      -DDISABLE_TCPCACHE=ON && \
    make && \
    make install

# Install CppDB
RUN mkdir -p /root/build/cppdb && \
    cd /root/build/cppdb && \
    cmake /root/src/cppdb/ \
      -DCMAKE_BUILD_TYPE=Release \
      -DCMAKE_INSTALL_PREFIX=/usr/local/cppdb \
      -DSQLITE_BACKEND_INTERNAL=ON \
      -DDISABLE_MYSQL=ON \
      -DDISABLE_PQ=ON \
      -DDISABLE_ODBC=ON && \
    make && \
    make install

# Install kennel2
RUN cd /root/src/wandbox/kennel2 && \
    ./autogen.sh && \
    cd /root/src/wandbox/kennel2 && \
    ./configure \
      --prefix=/usr/local/kennel2 \
      --with-cppcms=/usr/local/cppcms \
      --with-cppdb=/usr/local/cppdb && \
    make && \
    make install

ENV PATH $PATH:/usr/local/kennel2/bin
ENV LD_LIBRARY_PATH $LD_LIBRARY_PATH:/usr/local/cppcms/lib:/usr/local/cppdb/lib:/usr/local/kennel2/lib

# Run kennel2
CMD [ "kennel", "-c", "/usr/local/kennel2/etc/kennel.json" ]
