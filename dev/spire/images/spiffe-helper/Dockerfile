FROM busybox:1.32.0-glibc

ENV SPIFFE_HELPER_VERSION "0.5"

RUN wget -c https://github.com/spiffe/spiffe-helper/releases/download/$SPIFFE_HELPER_VERSION/spiffe-helper-$SPIFFE_HELPER_VERSION-Linux-x86_64.tar.gz -O - | tar -xz
COPY helper.conf /

ENTRYPOINT /spiffe-helper
