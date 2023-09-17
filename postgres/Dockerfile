FROM postgres:15-alpine AS pg_builder

ARG pg_spgist_hamming_ref=9fa70b08e0f0108de6a6673ce095c86a987d261d

RUN apk add --no-cache --virtual .build-deps \
        build-base \
        clang15-dev \
        llvm15-dev \
        make \
        wget

RUN wget -O pg-spgist_hamming.tar.gz https://github.com/fake-name/pg-spgist_hamming/archive/${pg_spgist_hamming_ref}.tar.gz \
    && mkdir -p /usr/src/pg-spgist_hamming \
    && tar -xzf pg-spgist_hamming.tar.gz -C /usr/src/pg-spgist_hamming --strip-components=1 \
    && rm pg-spgist_hamming.tar.gz

RUN cd /usr/src/pg-spgist_hamming/bktree \
    && make \
    && make install

FROM postgres:15-alpine

COPY --from=pg_builder /usr/local/lib/postgresql/bktree.so /usr/local/lib/postgresql/

COPY --from=pg_builder /usr/local/share/postgresql/extension/bktree* /usr/local/share/postgresql/extension/

ENV POSTGRES_USER=the-martian-bot

ENV POSTGRES_DB=the-martian-bot

CMD ["postgres", "-c", "config_file=/etc/postgresql/postgresql.conf"]
