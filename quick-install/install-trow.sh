#!/bin/sh

namespace='kube-public'
if [ -z $1 ]
then
	namespace=$1
fi
trow_dir=$(dirname $0)

tmp_file=$(mktemp)
sed "s/{{namespace}}/${namespace}/" ${trow_dir}/trow-tmpl.yaml > $tmp_file

kubelet -n $namespace apply -f $tmp_file

