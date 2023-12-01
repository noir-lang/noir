variable "DEPLOY_TAG" {
  type = string
}

variable "BOOTNODE_LISTEN_PORT" {
  type    = string
  default = 40500
}

variable "BOOTNODE_1_PRIVATE_KEY" {
  type = string
}

variable "BOOTNODE_2_PRIVATE_KEY" {
  type = string
}

variable "P2P_MIN_PEERS" {
  type    = string
  default = 50
}

variable "P2P_MAX_PEERS" {
  type    = string
  default = 100
}

variable "DOCKERHUB_ACCOUNT" {
  type = string
}
