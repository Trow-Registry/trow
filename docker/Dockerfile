# syntax=docker/dockerfile:1

FROM --platform=$TARGETPLATFORM debian:bookworm-slim

ARG TROW_BIN_LOC=trow

LABEL org.opencontainers.image.source="https://github.com/trow-registry/trow"
LABEL org.opencontainers.image.description="Caching Container Registry and Image Management for Kubernetes Clusters"
LABEL org.opencontainers.image.licenses="Apache-2.0"

RUN groupadd -r -g 2000 trow &&\
    useradd -r -g trow -u 2000 trow

# Note that certs are needed for proxying
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update &&\
    apt-get install -y --no-install-recommends openssl ca-certificates

RUN mkdir -p /data /certs &&\
    chown -R trow /data /certs
COPY ${TROW_BIN_LOC} /trow
USER trow
ENTRYPOINT ["/trow"]

ARG VCS_REF
ARG VCS_BRANCH
ARG DATE
ARG VERSION
ARG REPO
ARG TAG
ENV CREATED=$DATE
ENV VCS_REF=$VCS_REF
ENV VCS_BRANCH=$VCS_BRANCH
ENV VERSION=$VERSION

LABEL org.opencontainers.image.created=$DATE \
  org.opencontainers.image.authors="Container Solutions Labs" \
  org.opencontainers.image.url="https://trow.io" \
  org.opencontainers.image.source="https://github.com/trow-registry/trow" \
  org.opencontainers.image.version=$VERSION \
  org.opencontainers.image.revision=$VCS_REF \
  git.branch=$VCS_BRANCH \
  org.opencontainers.image.title="Trow Cluster Registry" \
  repository=$REPO \
  tag=$TAG
