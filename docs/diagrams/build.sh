#!/bin/bash

cat kubernetes_diagram.py | docker run -i --rm -v $PWD:/out gtramontina/diagrams:0.10.0
cat distribution_diagram.py | docker run -i --rm -v $PWD:/out gtramontina/diagrams:0.10.0
