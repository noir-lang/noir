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

variable "DEPLOY_TAG" {
  type = string
}

# S3 Bucket to store contract addresses
resource "aws_s3_bucket" "contract_addresses" {
  bucket = "aztec-${var.DEPLOY_TAG}-deployments"
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


variable "GAS_TOKEN_CONTRACT_ADDRESS" {
  type    = string
  default = ""
}

output "gas_token_contract_address" {
  value = var.GAS_TOKEN_CONTRACT_ADDRESS
}

variable "GAS_PORTAL_CONTRACT_ADDRESS" {
  type    = string
  default = ""
}

output "gas_portal_contract_address" {
  value = var.GAS_PORTAL_CONTRACT_ADDRESS
}
