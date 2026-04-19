# cross-builders/mips/Dockerfile
FROM ghcr.io/cross-rs/mips-unknown-linux-musl:main

ENV PG_VERSION=15.13
ENV MIPS_TARGET=mips-linux-muslsf
ENV STAGING=/opt/mips-musl-sysroot
ENV PATH=/usr/local/bin:$PATH

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    wget \
    make \
    gcc \
    g++ \
    perl \
    python3 \
    bison \
    flex \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Download and build PostgreSQL client library for MIPS musl
RUN wget -q https://ftp.postgresql.org/pub/source/v${PG_VERSION}/postgresql-${PG_VERSION}.tar.bz2 \
    && tar xjf postgresql-${PG_VERSION}.tar.bz2 \
    && cd postgresql-${PG_VERSION} \
    && ./configure \
        --host=${MIPS_TARGET} \
        --prefix=${STAGING}/usr \
        --without-readline \
        --without-zlib \
        --without-openssl \
        --without-gssapi \
        --without-ldap \
        --without-pam \
        --without-bonjour \
        --without-libxml \
        --without-libxslt \
        --without-python \
        --without-perl \
        --without-tcl \
        --disable-largefile \
        CC=${MIPS_TARGET}-gcc \
        CFLAGS="-Os" \
    && make -C src/interfaces/libpq \
    && make -C src/interfaces/libpq install \
    && make -C src/include install \
    && make -C src/bin/pg_config install \
    && cd / \
    && rm -rf postgresql-${PG_VERSION}*

# Create pkg-config file for cross-compilation
RUN mkdir -p ${STAGING}/usr/lib/pkgconfig && \
    cat > ${STAGING}/usr/lib/pkgconfig/libpq.pc << EOF
prefix=${STAGING}/usr
exec_prefix=\${prefix}
libdir=\${prefix}/lib
includedir=\${prefix}/include

Name: libpq
Description: PostgreSQL client library
Version: ${PG_VERSION}
Libs: -L\${libdir} -lpq
Cflags: -I\${includedir}
EOF

ENV PKG_CONFIG_PATH=${STAGING}/usr/lib/pkgconfig
ENV PKG_CONFIG_LIBDIR=${STAGING}/usr/lib/pkgconfig
ENV PKG_CONFIG_SYSROOT_DIR=${STAGING}
ENV MIPS_UNKNOWN_LINUX_MUSL_OPENSSL_DIR=${STAGING}/usr
ENV PQ_LIB_DIR=${STAGING}/usr/lib
