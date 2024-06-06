# Terraform to setup a prototype network of Aztec Boot Nodes in AWS
# It sets up 2 boot nodes with different ports/keys etc.
# Some duplication across the 2 defined services, could possibly
# be refactored to use modules as and when we build out infrastructure for real

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

data "terraform_remote_state" "aztec-network_iac" {
  backend = "s3"
  config = {
    bucket = "aztec-terraform"
    key    = "aztec-network/iac"
    region = "eu-west-2"
  }
}

locals {
  bootnode_keys  = [var.BOOTNODE_1_PRIVATE_KEY]
  bootnode_count = length(local.bootnode_keys)
}


resource "aws_cloudwatch_log_group" "p2p-bootstrap-log-group" {
  count             = local.bootnode_count
  name              = "/fargate/service/${var.DEPLOY_TAG}/p2p-bootstrap-${count.index + 1}"
  retention_in_days = 14
}

resource "aws_service_discovery_service" "p2p-bootstrap" {
  count = local.bootnode_count
  name  = "${var.DEPLOY_TAG}-p2p-bootstrap-${count.index + 1}"

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
    command = "${path.module}/servicediscovery-drain.sh ${self.id}"
  }
}

resource "aws_ecs_task_definition" "p2p-bootstrap" {
  count                    = local.bootnode_count
  family                   = "${var.DEPLOY_TAG}-p2p-bootstrap-${count.index + 1}"
  requires_compatibilities = ["FARGATE"]
  network_mode             = "awsvpc"
  cpu                      = "256"
  memory                   = "1024"
  execution_role_arn       = data.terraform_remote_state.setup_iac.outputs.ecs_task_execution_role_arn
  task_role_arn            = data.terraform_remote_state.aztec2_iac.outputs.cloudwatch_logging_ecs_role_arn

  container_definitions = <<DEFINITIONS
[
  {
    "name": "${var.DEPLOY_TAG}-p2p-bootstrap-${count.index + 1}",
    "image": "${var.DOCKERHUB_ACCOUNT}/aztec:${var.DEPLOY_TAG}",
    "command": ["start", "--p2p-bootstrap"],
    "essential": true,
    "memoryReservation": 776,
    "portMappings": [
      {
        "containerPort": ${var.BOOTNODE_LISTEN_PORT + count.index},
        "protocol": "udp"
      },
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
        "name": "P2P_UDP_LISTEN_ADDR",
        "value": "0.0.0.0:${var.BOOTNODE_LISTEN_PORT + count.index}"
      },
      {
        "name": "P2P_UDP_ANNOUNCE_ADDR",
        "value": "${data.terraform_remote_state.aztec-network_iac.outputs.p2p_eip}:${var.BOOTNODE_LISTEN_PORT + count.index}"
      },
      {
        "name": "PEER_ID_PRIVATE_KEY",
        "value": "${local.bootnode_keys[count.index]}"
      },
      {
        "name": "DEBUG",
        "value": "aztec:*,discv5:*"
      },
      {
        "name": "P2P_MIN_PEERS",
        "value": "${var.P2P_MIN_PEERS}"
      },
      {
        "name": "P2P_MAX_PEERS",
        "value": "${var.P2P_MAX_PEERS}"
      },
      {
        "name": "HTTP_PORT",
        "value": "80"
      }
    ],
    "logConfiguration": {
      "logDriver": "awslogs",
      "options": {
        "awslogs-group": "/fargate/service/${var.DEPLOY_TAG}/p2p-bootstrap-${count.index + 1}",
        "awslogs-region": "eu-west-2",
        "awslogs-stream-prefix": "ecs"
      }
    }
  }
]
DEFINITIONS
}

resource "aws_ecs_service" "p2p-bootstrap" {
  count                              = local.bootnode_count
  name                               = "${var.DEPLOY_TAG}-p2p-bootstrap-${count.index + 1}"
  cluster                            = data.terraform_remote_state.setup_iac.outputs.ecs_cluster_id
  launch_type                        = "FARGATE"
  desired_count                      = 1
  deployment_maximum_percent         = 100
  deployment_minimum_healthy_percent = 0
  platform_version                   = "1.4.0"

  network_configuration {
    subnets = [
      data.terraform_remote_state.setup_iac.outputs.subnet_az1_private_id
    ]
    security_groups = [data.terraform_remote_state.aztec-network_iac.outputs.p2p_security_group_id, data.terraform_remote_state.setup_iac.outputs.security_group_private_id]
  }

  service_registries {
    registry_arn   = aws_service_discovery_service.p2p-bootstrap[count.index].arn
    container_name = "${var.DEPLOY_TAG}-p2p-bootstrap-${count.index + 1}"
    container_port = 80
  }

  load_balancer {
    target_group_arn = aws_lb_target_group.p2p-bootstrap-target-group-udp[count.index].id
    container_name   = "${var.DEPLOY_TAG}-p2p-bootstrap-${count.index + 1}"
    container_port   = var.BOOTNODE_LISTEN_PORT + count.index
  }

  task_definition = aws_ecs_task_definition.p2p-bootstrap[count.index].family
}

# Create a target group for UDP traffic.
resource "aws_lb_target_group" "p2p-bootstrap-target-group-udp" {
  count       = local.bootnode_count
  name        = "p2p-bootstrap-${count.index + 1}-target-group-udp"
  port        = var.BOOTNODE_LISTEN_PORT + count.index
  protocol    = "UDP"
  target_type = "ip"
  vpc_id      = data.terraform_remote_state.setup_iac.outputs.vpc_id

  health_check {
    enabled             = true
    path                = "/health"
    port                = "80"
    interval            = 30
    timeout             = 10
    healthy_threshold   = 2
    unhealthy_threshold = 2
    matcher             = "200"
    protocol            = "HTTP"
  }
}

resource "aws_lb_listener" "p2p-bootstrap-udp-listener" {
  count             = local.bootnode_count
  load_balancer_arn = data.terraform_remote_state.aztec-network_iac.outputs.nlb_arn
  port              = var.BOOTNODE_LISTEN_PORT + count.index
  protocol          = "UDP"

  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.p2p-bootstrap-target-group-udp[count.index].arn
  }
}

resource "aws_security_group_rule" "allow-bootstrap-udp" {
  count             = local.bootnode_count
  type              = "ingress"
  from_port         = var.BOOTNODE_LISTEN_PORT + count.index
  to_port           = var.BOOTNODE_LISTEN_PORT + count.index
  protocol          = "udp"
  security_group_id = data.terraform_remote_state.aztec-network_iac.outputs.p2p_security_group_id
  cidr_blocks       = ["0.0.0.0/0"]
}

# Egress Rule for UDP Traffic
resource "aws_security_group_rule" "allow-bootstrap-udp-egress" {
  count             = local.bootnode_count
  type              = "egress"
  from_port         = var.BOOTNODE_LISTEN_PORT + count.index
  to_port           = var.BOOTNODE_LISTEN_PORT + count.index
  protocol          = "udp"
  security_group_id = data.terraform_remote_state.aztec-network_iac.outputs.p2p_security_group_id
  cidr_blocks       = ["0.0.0.0/0"]
}
