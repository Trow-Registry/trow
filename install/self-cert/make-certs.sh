#!/bin/bash

openssl req -config in.req -newkey rsa:4096 -nodes -sha256 -keyout domain.key -x509 -days 265 -out ca.crt
