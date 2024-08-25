{{/*
Expand the name of the chart.
*/}}
{{- define "aztec-network.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "aztec-network.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "aztec-network.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "aztec-network.labels" -}}
helm.sh/chart: {{ include "aztec-network.chart" . }}
{{ include "aztec-network.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "aztec-network.selectorLabels" -}}
app.kubernetes.io/name: {{ include "aztec-network.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{- define "aztec-network.ethereumHost" -}}
http://{{ include "aztec-network.fullname" . }}-ethereum.{{ .Release.Namespace }}:{{ .Values.ethereum.service.port }}
{{- end -}}

{{- define "aztec-network.pxeUrl" -}}
http://{{ include "aztec-network.fullname" . }}-pxe.{{ .Release.Namespace }}:{{ .Values.pxe.service.port }}
{{- end -}}
