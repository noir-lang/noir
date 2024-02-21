terraform {
  backend "s3" {
    bucket = "aztec-terraform"
    region = "eu-west-2"
  }
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "3.74.2"
    }
  }
}

variable "ROLLUP_CONTRACT_ADDRESS" {
  type    = string
  default = ""
}

output "rollup_contract_address" {
  value = var.ROLLUP_CONTRACT_ADDRESS
}

variable "AVAILABILITY_ORACLE_CONTRACT_ADDRESS" {
  type    = string
  default = ""
}

output "availability_oracle_contract_address" {
  value = var.AVAILABILITY_ORACLE_CONTRACT_ADDRESS
}

variable "REGISTRY_CONTRACT_ADDRESS" {
  type    = string
  default = ""
}

output "registry_contract_address" {
  value = var.REGISTRY_CONTRACT_ADDRESS
}

variable "INBOX_CONTRACT_ADDRESS" {
  type    = string
  default = ""
}

output "inbox_contract_address" {
  value = var.INBOX_CONTRACT_ADDRESS
}

variable "OUTBOX_CONTRACT_ADDRESS" {
  type    = string
  default = ""
}

output "outbox_contract_address" {
  value = var.OUTBOX_CONTRACT_ADDRESS
}

variable "CONTRACT_DEPLOYMENT_EMITTER_ADDRESS" {
  type    = string
  default = ""
}

output "contract_deployment_emitter_address" {
  value = var.CONTRACT_DEPLOYMENT_EMITTER_ADDRESS
}
