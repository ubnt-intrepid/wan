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
      libboost-dev \
      libboost-program-options-dev \
      libboost-system-dev \
      libcap-dev \
      libgcrypt11-dev \
      libicu-dev \
      libpcre3-dev \
      libsqlite3-dev \
      make \
      python \
      rsync \
      supervisor \
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

# Install cattleshed
RUN cd /root/src/wandbox/cattleshed && \
    autoreconf -i && \
    ./configure \
      --prefix=/usr/local && \
    make && \
    make install

# Install kennel2
RUN cd /root/src/wandbox/kennel2 && \
    ./autogen.sh && \
    ./configure \
      --prefix=/usr/local \
      --with-cppcms=/usr/local/cppcms \
      --with-cppdb=/usr/local/cppdb && \
    make && \
    make install

RUN setcap cap_sys_admin,cap_sys_chroot,cap_mknod,cap_net_admin,cap_sys_rawio,cap_sys_module=p \
      /usr/local/bin/cattlegrid

# Overwrite configuration
COPY kennel.json        /usr/local/etc/kennel.json
COPY cattleshed.conf    /usr/local/etc/cattleshed.conf
COPY compilers.default  /usr/local/etc/cattleshed.conf.d/compilers.default
COPY supervisord.conf   /etc/supervisord.conf

EXPOSE 3500

RUN useradd -ms /bin/bash vagrant
RUN useradd -ms /bin/bash jail

RUN mkdir -p /var/lib/kennel
RUN touch /var/lib/kennel/kennel_production.sqlite
RUN chown -R vagrant:vagrant /var/lib/kennel

RUN mkdir -p /var/log/wandbox/ran
RUN chown -R vagrant:vagrant /var/log/wandbox/ran

# Run services
CMD [ "/usr/bin/supervisord" ]
