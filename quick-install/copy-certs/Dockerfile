FROM debian:jessie

RUN apt-get update && apt-get install -y \
      ed curl \
    && rm -rf /var/lib/apt/lists/*

COPY job.sh /
RUN chmod +x /job.sh
ENTRYPOINT /job.sh
