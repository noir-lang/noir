variable "DEPLOY_TAG" {
  type = string
}

variable "RPC_URL" {
  type    = string
  default = "testnet"
}

variable "API_KEY" {
  type = string
}

variable "API_PREFIX" {
  type    = string
  default = ""
}

variable "CHAIN_ID" {
  type = string
}

variable "FAUCET_PRIVATE_KEY" {
  type = string
}

variable "DOCKERHUB_ACCOUNT" {
  type = string
}
