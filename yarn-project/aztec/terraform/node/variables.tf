variable "DEPLOY_TAG" {
  type = string
}

variable "IMAGE_TAG" {
  type = string
}

variable "API_KEY" {
  type = string
}

variable "SEQUENCER_PRIVATE_KEYS" {
  type = list(string)
}

variable "NODE_P2P_PRIVATE_KEYS" {
  type = list(string)
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

variable "DOCKERHUB_ACCOUNT" {
  type = string
}

variable "SEQ_MAX_TX_PER_BLOCK" {
  type    = string
  default = 64
}

variable "SEQ_MIN_TX_PER_BLOCK" {
  type    = string
  default = 0
}

variable "SEQ_MAX_SECONDS_BETWEEN_BLOCKS" {
  type    = string
  default = 60
}

variable "SEQ_MIN_SECONDS_BETWEEN_BLOCKS" {
  type    = string
  default = 30
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
  default = false
}
