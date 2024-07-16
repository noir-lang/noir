variable "DEPLOY_TAG" {
  type = string
}

variable "API_KEY" {
  type = string
}

variable "SEQ_1_PUBLISHER_PRIVATE_KEY" {
  type = string
}

variable "SEQ_2_PUBLISHER_PRIVATE_KEY" {
  type = string
}

variable "L1_CHAIN_ID" {
  type    = string
  default = 677692
}

variable "NODE_P2P_TCP_PORT" {
  type    = number
  default = 40400
}

variable "NODE_P2P_UDP_PORT" {
  type    = number
  default = 40300
}

variable "NODE_1_PRIVATE_KEY" {
  type    = string
  default = ""
}

variable "NODE_2_PRIVATE_KEY" {
  type    = string
  default = ""
}

variable "DOCKERHUB_ACCOUNT" {
  type = string
}

variable "SEQ_MAX_TX_PER_BLOCK" {
  type    = string
  default = 64
}

variable "SEQ_MIN_TX_PER_BLOCK" {
  type    = string
  default = 1
}

variable "P2P_MIN_PEERS" {
  type    = string
  default = 5
}

variable "P2P_MAX_PEERS" {
  type    = string
  default = 100
}

variable "P2P_ENABLED" {
  type    = bool
  default = true
}

variable "PROVING_ENABLED" {
  type    = bool
  default = true
}
