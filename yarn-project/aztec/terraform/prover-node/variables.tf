variable "DEPLOY_TAG" {
  type = string
}

variable "IMAGE_TAG" {
  type    = string
  default = "latest"
}

variable "API_KEY" {
  type = string
}

variable "PROVER_PRIVATE_KEYS" {
  type = list(string)
}

variable "NODE_P2P_PRIVATE_KEYS" {
  type = list(string)
}

variable "L1_CHAIN_ID" {
  type = string
}

variable "NODE_P2P_TCP_PORT" {
  type = number
}

variable "NODE_P2P_UDP_PORT" {
  type = number
}

variable "DOCKERHUB_ACCOUNT" {
  type = string
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
  default = false
}

variable "P2P_TX_POOL_KEEP_PROVEN_FOR" {
  type    = number
  default = 64
}

variable "PROVING_ENABLED" {
  type    = bool
  default = false
}

variable "PROVER_NODE_MAX_PENDING_JOBS" {
  type    = number
  default = 16
}

variable "BOOTSTRAP_NODES" {
  type    = string
  default = ""
}

variable "PROVER_NODE_LB_RULE_PRIORITY" {
  type = number
}
