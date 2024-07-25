variable "DOCKERHUB_ACCOUNT" {
  type = string
}

variable "GRAFANA_CLIENT_ID" {
  type = string
}

variable "GRAFANA_CLIENT_SECRET" {
  type = string
}

variable "IMAGE_TAG" {
  type = string
  default = "latest"
}
