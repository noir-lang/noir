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

provider "aws" {
  profile = "default"
  region  = "eu-west-2"
}

resource "aws_service_discovery_service" "aztec_otel" {
  name = "aztec-otel"

  health_check_custom_config {
    failure_threshold = 1
  }

  dns_config {
    namespace_id = data.terraform_remote_state.setup_iac.outputs.local_service_discovery_id

    dns_records {
      ttl  = 60
      type = "A"
    }

    routing_policy = "MULTIVALUE"
  }
}

# Define task definition and service.
resource "aws_ecs_task_definition" "aztec_otel" {
  family                   = "aztec-otel"
  requires_compatibilities = ["FARGATE"]
  network_mode             = "awsvpc"
  cpu                      = "2048"
  memory                   = "4096"
  execution_role_arn       = data.terraform_remote_state.setup_iac.outputs.ecs_task_execution_role_arn
  task_role_arn            = data.terraform_remote_state.aztec2_iac.outputs.cloudwatch_logging_ecs_role_arn

  container_definitions = <<DEFINITIONS
[
  {
    "name": "aztec-otel",
    "image": "${var.DOCKERHUB_ACCOUNT}/aztec-otel:${var.IMAGE_TAG}",
    "essential": true,
    "memoryReservation": 256,
    "portMappings": [
      {
        "containerPort": 8888
      },
      {
        "containerPort": 8889
      },
      {
        "containerPort": 4318
      }
    ],
    "logConfiguration": {
      "logDriver": "awslogs",
      "options": {
        "awslogs-group": "/fargate/service/aztec-otel",
        "awslogs-region": "eu-west-2",
        "awslogs-stream-prefix": "ecs"
      }
    }
  }
]
DEFINITIONS
}

data "aws_ecs_task_definition" "aztec_otel" {
  task_definition = aws_ecs_task_definition.aztec_otel.family
}

resource "aws_ecs_service" "aztec_otel" {
  name                               = "aztec-otel"
  cluster                            = data.terraform_remote_state.setup_iac.outputs.ecs_cluster_id
  launch_type                        = "FARGATE"
  desired_count                      = "1"
  deployment_maximum_percent         = 100
  deployment_minimum_healthy_percent = 0
  platform_version                   = "1.4.0"

  network_configuration {
    subnets = [
      data.terraform_remote_state.setup_iac.outputs.subnet_az1_private_id,
      data.terraform_remote_state.setup_iac.outputs.subnet_az2_private_id
    ]
    security_groups = [data.terraform_remote_state.setup_iac.outputs.security_group_private_id]
  }

  service_registries {
    registry_arn = aws_service_discovery_service.aztec_otel.arn
  }

  task_definition = "${aws_ecs_task_definition.aztec_otel.family}:${max(aws_ecs_task_definition.aztec_otel.revision, data.aws_ecs_task_definition.aztec_otel.revision)}"
}

# Logs
resource "aws_cloudwatch_log_group" "aztec_otel_logs" {
  name              = "/fargate/service/aztec-otel"
  retention_in_days = "14"
}
