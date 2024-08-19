variable "DEPLOY_TAG" {
  type = string
}

variable "DOCKERHUB_ACCOUNT" {
  type = string
}

variable "API_KEY" {
  type = string
}

variable "BOT_API_KEY" {
  type = string
}

variable "BOT_PRIVATE_KEY" {
  type = string
}

variable "BOT_NO_START" {
  type = string
}

variable "BOT_PRIVATE_TRANSFERS_PER_TX" {
  type = string
}

variable "BOT_PUBLIC_TRANSFERS_PER_TX" {
  type = string
}
variable "LOG_LEVEL" {
  type    = string
  default = "verbose"
}

variable "BOT_TX_INTERVAL_SECONDS" {
  type    = string
  default = "300"
}

variable "BOT_TX_MINED_WAIT_SECONDS" {
  type = string
}

variable "BOT_FOLLOW_CHAIN" {
  type = string
}

variable "PROVING_ENABLED" {
  type    = bool
  default = false
}

variable "BOT_COUNT" {
  type    = string
  default = "1"
}

variable "BOT_FLUSH_SETUP_TRANSACTIONS" {
  type    = bool
  default = false
}

variable "BOT_MAX_PENDING_TXS" {
  type    = number
  default = 1
}
