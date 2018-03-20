FROM cfssl/cfssl

RUN curl -LO https://storage.googleapis.com/kubernetes-release/release/v1.9.0/bin/linux/amd64/kubectl
RUN chmod +x ./kubectl 
RUN mv ./kubectl /usr/local/bin/kubectl
RUN mkdir /certs
WORKDIR /certs
COPY init.sh /certs/
COPY req.json /certs/
COPY csr.yaml /certs/

ENTRYPOINT [""]
CMD ["./init.sh"]
