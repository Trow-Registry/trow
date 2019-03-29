FROM cfssl/cfssl

# This all needs to be cleaned up and improved for provenance
# Ideally we'd move this to application code, but that's a fair bit of work

RUN curl -LO https://storage.googleapis.com/kubernetes-release/release/v1.14.0/bin/linux/amd64/kubectl
RUN chmod +x ./kubectl 
RUN mv ./kubectl /usr/local/bin/kubectl
RUN apt update
RUN apt install -y dnsutils jq
COPY init.sh /

ENTRYPOINT [""]
CMD ["/init.sh"]
