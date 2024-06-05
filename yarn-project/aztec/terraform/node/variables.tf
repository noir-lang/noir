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
  default = 50
}

variable "P2P_MAX_PEERS" {
  type    = string
  default = 100
}

variable "P2P_ENABLED" {
  type = bool
}

variable "AVAILABILITY_ORACLE_CONTRACT_ADDRESS" { type = string }
variable "ROLLUP_CONTRACT_ADDRESS" { type = string }
variable "REGISTRY_CONTRACT_ADDRESS" { type = string }
variable "INBOX_CONTRACT_ADDRESS" { type = string }
variable "OUTBOX_CONTRACT_ADDRESS" { type = string }
variable "GAS_TOKEN_CONTRACT_ADDRESS" { type = string }
variable "GAS_PORTAL_CONTRACT_ADDRESS" { type = string }
variable "AGENTS_PER_SEQUENCER" { type = string }
variable "PROVING_ENABLED" {
  type    = bool
  default = true
}

variable "IMAGE_TAG" {
  type = string
}

variable "FULL_IMAGE" {
  type = string
}
