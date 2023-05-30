{{/* vim: set filetype=mustache: */}}
{{/*
Expand the name of the chart.
*/}}
{{- define "trow.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "trow.fullname" -}}
{{- if .Values.fullnameOverride -}}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" -}}
{{- else -}}
{{- $name := default .Chart.Name .Values.nameOverride -}}
{{- if contains $name .Release.Name -}}
{{- .Release.Name | trunc 63 | trimSuffix "-" -}}
{{- else -}}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" -}}
{{- end -}}
{{- end -}}
{{- end -}}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "trow.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{/*
Common labels
*/}}
{{- define "trow.labels" -}}
helm.sh/chart: {{ include "trow.chart" . }}
{{ include "trow.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end -}}

{{/*
Common selector labels
*/}}
{{- define "trow.selectorLabels" -}}
app.kubernetes.io/part-of: {{ include "trow.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end -}}

{{/*
Registry labels
*/}}
{{- define "registry.labels" -}}
{{ include "trow.labels" . }}
{{ include "registry.selectorLabels" . }}
{{- end -}}

{{/*
Registry selector labels
*/}}
{{- define "registry.selectorLabels" -}}
app.kubernetes.io/name: {{ include "trow.name" . }}-registry
{{- end -}}

{{/*
Webhook labels
*/}}
{{- define "webhook.labels" -}}
{{ include "trow.labels" . }}
{{ include "webhook.selectorLabels" . }}
{{- end -}}

{{/*
Webhook selector labels
*/}}
{{- define "webhook.selectorLabels" -}}
app.kubernetes.io/name: {{ include "trow.name" . }}-webhook
{{- end -}}
