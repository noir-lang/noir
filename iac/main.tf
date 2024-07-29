terraform {
  backend "s3" {
    bucket = "aztec-terraform"
    key    = "aztec-network/iac"
    region = "eu-west-2"
  }
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "3.74.2"
    }
  }
}

provider "aws" {
  profile = "default"
  region  = "eu-west-2"
}

data "terraform_remote_state" "setup_iac" {
  backend = "s3"
  config = {
    bucket = "aztec-terraform"
    key    = "setup/setup-iac"
    region = "eu-west-2"
  }
}

data "terraform_remote_state" "aztec2_iac" {
  backend = "s3"
  config = {
    bucket = "aztec-terraform"
    key    = "aztec2/iac"
    region = "eu-west-2"
  }
}

# Allocate Elastic IPs for each subnet
resource "aws_eip" "aztec_network_p2p_eip" {
  vpc = true
}

# Create our load balancer.
resource "aws_lb" "aztec-network" {
  name               = "aztec-network"
  internal           = false
  load_balancer_type = "network"
  security_groups = [
    data.terraform_remote_state.setup_iac.outputs.security_group_public_id, aws_security_group.security-group-p2p.id
  ]

  subnet_mapping {
    subnet_id     = data.terraform_remote_state.setup_iac.outputs.subnet_az1_id
    allocation_id = aws_eip.aztec_network_p2p_eip.id
  }

  # No EIP for the second subnet, so it will use a dynamic IP.
  subnet_mapping {
    subnet_id = data.terraform_remote_state.setup_iac.outputs.subnet_az2_id
  }

  access_logs {
    bucket  = "aztec-logs"
    prefix  = "aztec-network-nlb-logs"
    enabled = false
  }

  tags = {
    Name = "aztec-network"
  }
}

resource "aws_security_group" "security-group-p2p" {
  name        = "security-group-p2p"
  description = "Allow inbound p2p traffic"
  vpc_id      = data.terraform_remote_state.setup_iac.outputs.vpc_id

  tags = {
    Name = "allow-p2p"
  }
}

# static.aztec.network resources
resource "aws_s3_bucket" "contract_addresses" {
  bucket = "static.aztec.network"

  website {
    index_document = "index.html"
  }
}

resource "aws_s3_bucket_public_access_block" "addresses_public_access" {
  bucket = aws_s3_bucket.contract_addresses.id

  block_public_acls       = false
  block_public_policy     = false
  ignore_public_acls      = false
  restrict_public_buckets = false
}

resource "aws_s3_bucket_policy" "addresses_bucket_policy" {
  bucket = aws_s3_bucket.contract_addresses.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect    = "Allow"
        Principal = "*"
        Action    = "s3:GetObject"
        Resource  = "arn:aws:s3:::${aws_s3_bucket.contract_addresses.id}/*"
      }
    ]
  })
}

resource "aws_route53_record" "static" {
  zone_id = data.terraform_remote_state.aztec2_iac.outputs.aws_route53_zone_id
  name    = "static.aztec.network"
  type    = "A"

  alias {
    name                   = aws_s3_bucket.contract_addresses.website_domain
    zone_id                = aws_s3_bucket.contract_addresses.hosted_zone_id
    evaluate_target_health = true
  }
}
