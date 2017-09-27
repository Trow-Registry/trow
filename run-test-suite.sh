#!/usr/bin/env bash

# we could add a random string on the end of vars to avoid collisions
container="lyc-reg-test" 
network="lyc-reg-test-net"

docker network create ${network}
docker run -d --name ${container} --net ${network} lycaon
docker run --rm --net ${network} -e BASE_URL="http://${container}:8000/" amouat/registry-test-suite
code=$?
docker stop ${container}
docker rm ${container}
docker network rm ${network}
exit $code
