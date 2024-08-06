variable "DEPLOY_TAG" {
  type = string
}

variable "IMAGE_TAG" {
  type    = string
  default = "latest"
}

variable "AGENTS_PER_PROVER" {
  type    = string
  default = 1
}

variable "PROVING_ENABLED" {
  type    = bool
  default = false
}

variable "DOCKERHUB_ACCOUNT" {
  type = string
}

variable "API_KEY" {
  type = string
}
