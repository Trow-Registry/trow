#!/usr/bin/env sh
set -ex

#
# Script to build a values.schema.json for an umbrella chart, merging in the values.yaml files of all subcharts.
#

HELM_PLUG_SCHEMA_URL="https://github.com/losisin/helm-values-schema-json"
HELM_PLUG_SCHEMA_VERSION="2.3.1"

if [ "$HOME" = "/" ]; then
    echo "Workaround HOME set to / when using rootful docker with prek (that sets --user)"
    export HOME=/tmp/
fi

if [ "$(helm plugin list | awk '/^schema[[:space:]]/ {print $2}')" != "$HELM_PLUG_SCHEMA_VERSION" ]; then
    helm plugin uninstall schema || true
    helm plugin install --verify=false --version "v${HELM_PLUG_SCHEMA_VERSION}" "$HELM_PLUG_SCHEMA_URL"
fi

for values_yaml in "${@}"; do
    chart_dir="$(dirname "$values_yaml")"
    (cd "$chart_dir" && helm schema --use-helm-docs)
done
