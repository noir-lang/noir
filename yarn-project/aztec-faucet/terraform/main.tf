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

# Define provider and region
provider "aws" {
  region = "eu-west-2"
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

locals {
  api_prefix = "/${var.DEPLOY_TAG}/aztec-faucet/${var.API_KEY}"
  rpc_url    = "https://${var.DEPLOY_TAG}-mainnet-fork.aztec.network:8545/${var.API_KEY}"
}


resource "aws_cloudwatch_log_group" "aztec-faucet" {
  name              = "/fargate/service/${var.DEPLOY_TAG}/aztec-faucet"
  retention_in_days = 14
}

resource "aws_service_discovery_service" "aztec-faucet" {
  name = "${var.DEPLOY_TAG}-faucet"

  health_check_custom_config {
    failure_threshold = 1
  }

  dns_config {
    namespace_id = data.terraform_remote_state.setup_iac.outputs.local_service_discovery_id

    dns_records {
      ttl  = 60
      type = "A"
    }

    dns_records {
      ttl  = 60
      type = "SRV"
    }

    routing_policy = "MULTIVALUE"
  }

  # Terraform just fails if this resource changes and you have registered instances.
  provisioner "local-exec" {
    when    = destroy
    command = "${path.module}/../servicediscovery-drain.sh ${self.id}"
  }
}

# Define task definition and service.
resource "aws_ecs_task_definition" "aztec-faucet" {
  family                   = "${var.DEPLOY_TAG}-aztec-faucet"
  requires_compatibilities = ["FARGATE"]
  network_mode             = "awsvpc"
  cpu                      = "2048"
  memory                   = "4096"
  execution_role_arn       = data.terraform_remote_state.setup_iac.outputs.ecs_task_execution_role_arn
  task_role_arn            = data.terraform_remote_state.aztec2_iac.outputs.cloudwatch_logging_ecs_role_arn
  container_definitions    = <<DEFINITIONS
[
  {
    "name": "${var.DEPLOY_TAG}-aztec-faucet",
    "image": "${var.DOCKERHUB_ACCOUNT}/aztec-faucet:${var.DEPLOY_TAG}",
    "essential": true,
    "memoryReservation": 3776,
    "portMappings": [
      {
        "containerPort": 80
      }
    ],
    "environment": [
      {
        "name": "NODE_ENV",
        "value": "production"
      },
      {
        "name": "FAUCET_PORT",
        "value": "80"
      },
      {
        "name": "DEBUG",
        "value": "aztec:*"
      },
      {
        "name": "RPC_URL",
        "value": "${local.rpc_url}"
      },
      {
        "name": "API_PREFIX",
        "value": "${local.api_prefix}"
      },
      {
        "name": "L1_CHAIN_ID",
        "value": "${var.L1_CHAIN_ID}"
      },
      {
        "name": "PRIVATE_KEY",
        "value": "${var.FAUCET_PRIVATE_KEY}"
      },
      {
        "name": "INTERVAL",
        "value": "86400"
      },
      {
        "name": "ETH_AMOUNT",
        "value": "1.0"
      },
      {
        "name": "FAUCET_ACCOUNT_INDEX",
        "value": "${var.FAUCET_ACCOUNT_INDEX}"
      },
      {
        "name": "FORK_MNEMONIC",
        "value": "${var.FORK_MNEMONIC}"
      },
      {
        "name": "EXTRA_ASSETS",
        "value": "fee_juice:${var.GAS_TOKEN_CONTRACT_ADDRESS},dev_coin:${var.DEV_COIN_CONTRACT_ADDRESS}"
      },
      {
        "name": "EXTRA_ASSET_AMOUNT",
        "value": "1000000000"
      }
    ],
    "logConfiguration": {
      "logDriver": "awslogs",
      "options": {
        "awslogs-group": "/fargate/service/${var.DEPLOY_TAG}/aztec-faucet",
        "awslogs-region": "eu-west-2",
        "awslogs-stream-prefix": "ecs"
      }
    }
  }
]
DEFINITIONS
}

resource "aws_ecs_service" "aztec-faucet" {
  name                               = "${var.DEPLOY_TAG}-aztec-faucet"
  cluster                            = data.terraform_remote_state.setup_iac.outputs.ecs_cluster_id
  launch_type                        = "FARGATE"
  desired_count                      = 1
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

  load_balancer {
    target_group_arn = aws_alb_target_group.aztec-faucet.arn
    container_name   = "${var.DEPLOY_TAG}-aztec-faucet"
    container_port   = 80
  }

  service_registries {
    registry_arn   = aws_service_discovery_service.aztec-faucet.arn
    container_name = "${var.DEPLOY_TAG}-aztec-faucet"
    container_port = 80
  }

  task_definition = aws_ecs_task_definition.aztec-faucet.family
}

# Configure ALB to route /aztec-faucet to server.
resource "aws_alb_target_group" "aztec-faucet" {
  name                 = "${var.DEPLOY_TAG}-faucet"
  port                 = 80
  protocol             = "HTTP"
  target_type          = "ip"
  vpc_id               = data.terraform_remote_state.setup_iac.outputs.vpc_id
  deregistration_delay = 5

  health_check {
    path                = "${local.api_prefix}/status"
    matcher             = "200"
    interval            = 10
    healthy_threshold   = 2
    unhealthy_threshold = 5
    timeout             = 5
  }

  tags = {
    name = "${var.DEPLOY_TAG}-faucet"
  }
}

resource "aws_lb_listener_rule" "api-1" {
  listener_arn = data.terraform_remote_state.aztec2_iac.outputs.alb_listener_arn
  priority     = 600

  action {
    type             = "forward"
    target_group_arn = aws_alb_target_group.aztec-faucet.arn
  }

  condition {
    path_pattern {
      values = ["/${var.DEPLOY_TAG}/aztec-faucet*"]
    }
  }
}
