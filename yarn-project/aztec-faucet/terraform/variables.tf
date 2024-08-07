variable "DEPLOY_TAG" {
  type = string
}

variable "IMAGE_TAG" {
  type = string
}

variable "API_KEY" {
  type = string
}

variable "L1_CHAIN_ID" {
  type = string
}

variable "FAUCET_PRIVATE_KEY" {
  type    = string
  default = ""
}

variable "DOCKERHUB_ACCOUNT" {
  type = string
}

variable "FORK_MNEMONIC" {
  type = string
}

variable "FAUCET_ACCOUNT_INDEX" {
  type = string
}

variable "FEE_JUICE_CONTRACT_ADDRESS" {
  type = string
}

variable "DEV_COIN_CONTRACT_ADDRESS" {
  type = string
}

variable "FAUCET_LB_RULE_PRIORITY" {
  type    = number
  default = 600
}
