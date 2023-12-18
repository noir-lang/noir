terraform {
  backend "s3" {
    bucket = "aztec-terraform"
    region = "eu-west-2"
    key    = "aztec-up"
  }
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "5.29.0"
    }
  }
}

# Define provider and region
provider "aws" {
  region = "eu-west-2"
}

data "terraform_remote_state" "aztec2_iac" {
  backend = "s3"
  config = {
    bucket = "aztec-terraform"
    key    = "aztec2/iac"
    region = "eu-west-2"
  }
}

# Create the website S3 bucket
resource "aws_s3_bucket" "install_bucket" {
  bucket = "install.aztec.network"
}

resource "aws_s3_bucket_website_configuration" "website_bucket" {
  bucket = aws_s3_bucket.install_bucket.id

  index_document {
    suffix = "aztec-install"
  }
}

resource "aws_s3_bucket_public_access_block" "install_bucket_public_access" {
  bucket = aws_s3_bucket.install_bucket.id

  block_public_acls       = false
  ignore_public_acls      = false
  block_public_policy     = false
  restrict_public_buckets = false
}

resource "aws_s3_bucket_policy" "install_bucket_policy" {
  bucket = aws_s3_bucket.install_bucket.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect    = "Allow"
        Principal = "*"
        Action    = "s3:GetObject"
        Resource  = "arn:aws:s3:::${aws_s3_bucket.install_bucket.id}/*"
      }
    ]
  })
}

# Upload files to s3 bucket if changes were detected
resource "null_resource" "upload_public_directory" {
  triggers = {
    always_run = "${timestamp()}"
  }

  provisioner "local-exec" {
    command = "aws s3 sync ../bin s3://${aws_s3_bucket.install_bucket.id}/"
  }
}

resource "aws_route53_record" "subdomain_record" {
  zone_id = data.terraform_remote_state.aztec2_iac.outputs.aws_route53_zone_id
  name    = "install.aztec.network"
  type    = "A"

  alias {
    name                   = "${aws_s3_bucket_website_configuration.website_bucket.website_domain}"
    zone_id                = "${aws_s3_bucket.install_bucket.hosted_zone_id}"
    evaluate_target_health = true
  }
}
