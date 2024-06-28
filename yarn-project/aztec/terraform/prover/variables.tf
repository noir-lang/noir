variable "DEPLOY_TAG" {
  type = string
}

variable "AGENTS_PER_SEQUENCER" {
  type    = string
  default = 1
}

variable "PROVING_ENABLED" {
  type    = bool
  default = true
}

variable "DOCKERHUB_ACCOUNT" {
  type = string
}
