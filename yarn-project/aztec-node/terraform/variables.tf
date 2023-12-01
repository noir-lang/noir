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

variable "CHAIN_ID" {
  type    = string
  default = 31337
}

variable "BOOTNODE_LISTEN_PORT" {
  type    = number
  default = 40500
}

variable "BOOTNODE_1_PEER_ID" {
  type = string
}

variable "BOOTNODE_2_PEER_ID" {
  type = string
}

variable "NODE_TCP_PORT" {
  type    = number
  default = 40400
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
  default = 50
}

variable "P2P_MAX_PEERS" {
  type    = string
  default = 100
}
