#!/bin/bash

# The debug build is currently the default build
docker build -f Dockerfile.debug -t containersol/trow:default ../
