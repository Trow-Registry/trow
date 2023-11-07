#!/usr/bin/env bash

cat kubernetes_diagram.py | docker run -i --rm -v $PWD:/out gtramontina/diagrams:0.10.0
