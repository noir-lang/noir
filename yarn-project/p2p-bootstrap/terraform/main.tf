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


resource "aws_cloudwatch_log_group" "aztec-bootstrap-1-log-group" {
  name              = "/fargate/service/${var.DEPLOY_TAG}/aztec-bootstrap-1"
  retention_in_days = 14
}

resource "aws_service_discovery_service" "aztec-bootstrap-1" {
  name = "${var.DEPLOY_TAG}-aztec-bootstrap-1"

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

resource "aws_ecs_task_definition" "aztec-bootstrap-1" {
  family                   = "${var.DEPLOY_TAG}-aztec-bootstrap-1"
  requires_compatibilities = ["FARGATE"]
  network_mode             = "awsvpc"
  cpu                      = "2048"
  memory                   = "4096"
  execution_role_arn       = data.terraform_remote_state.setup_iac.outputs.ecs_task_execution_role_arn
  task_role_arn            = data.terraform_remote_state.aztec2_iac.outputs.cloudwatch_logging_ecs_role_arn

  container_definitions = <<DEFINITIONS
[
  {
    "name": "${var.DEPLOY_TAG}-aztec-bootstrap-1",
    "image": "${var.ECR_URL}/p2p-bootstrap:aztec3-packages-prod",
    "essential": true,
    "command": ["start"],
    "memoryReservation": 3776,
    "portMappings": [
      {
        "containerPort": ${var.BOOTNODE_1_LISTEN_PORT}
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
        "name": "P2P_TCP_LISTEN_PORT",
        "value": "${var.BOOTNODE_1_LISTEN_PORT}"
      },
      {
        "name": "P2P_TCP_LISTEN_IP",
        "value": "0.0.0.0"
      },
      {
        "name": "PEER_ID_PRIVATE_KEY",
        "value": "${var.BOOTNODE_1_PRIVATE_KEY}"
      },
      {
        "name": "DEBUG",
        "value": "aztec:*"
      }
    ],
    "logConfiguration": {
      "logDriver": "awslogs",
      "options": {
        "awslogs-group": "/fargate/service/${var.DEPLOY_TAG}/aztec-bootstrap-1",
        "awslogs-region": "eu-west-2",
        "awslogs-stream-prefix": "ecs"
      }
    }
  }
]
DEFINITIONS
}

resource "aws_ecs_service" "aztec-bootstrap-1" {
  name                               = "${var.DEPLOY_TAG}-aztec-bootstrap-1"
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
    security_groups = [data.terraform_remote_state.aztec-network_iac.outputs.p2p_security_group_id, data.terraform_remote_state.setup_iac.outputs.security_group_private_id]
  }

  service_registries {
    registry_arn   = aws_service_discovery_service.aztec-bootstrap-1.arn
    container_name = "${var.DEPLOY_TAG}-aztec-bootstrap-1"
    container_port = 80
  }

  load_balancer {
    target_group_arn = aws_lb_target_group.aztec-bootstrap-1-target-group.id
    container_name   = "${var.DEPLOY_TAG}-aztec-bootstrap-1"
    container_port   = var.BOOTNODE_1_LISTEN_PORT
  }

  task_definition = aws_ecs_task_definition.aztec-bootstrap-1.family
}

resource "aws_lb_target_group" "aztec-bootstrap-1-target-group" {
  name        = "aztec-bootstrap-1-target-group"
  port        = var.BOOTNODE_1_LISTEN_PORT
  protocol    = "TCP"
  target_type = "ip"
  vpc_id      = data.terraform_remote_state.setup_iac.outputs.vpc_id

  health_check {
    protocol            = "TCP"
    interval            = 10
    healthy_threshold   = 2
    unhealthy_threshold = 2
    port                = var.BOOTNODE_1_LISTEN_PORT
  }
}

resource "aws_security_group_rule" "allow-bootstrap-1-tcp" {
  type              = "ingress"
  from_port         = var.BOOTNODE_1_LISTEN_PORT
  to_port           = var.BOOTNODE_1_LISTEN_PORT
  protocol          = "tcp"
  cidr_blocks       = ["0.0.0.0/0"]
  security_group_id = data.terraform_remote_state.aztec-network_iac.outputs.p2p_security_group_id
}

## Commented out here and setup manually as terraform (or the aws provider version we are using) has a bug
## NLB listeners can't have a 'weight' property defined. You will see there isn't one here but that doesn't
## stop it trying to automatically specify one and giving an error

# resource "aws_lb_listener" "aztec-bootstrap-1-tcp-listener" {
#   load_balancer_arn = data.terraform_remote_state.aztec-network_iac.outputs.nlb_arn
#   port              = "${var.BOOTNODE_1_LISTEN_PORT}"
#   protocol          = "TCP"

#   tags = {
#     name = "aztec-bootstrap-1-target-group"
#   }

#   default_action {
#     type = "forward"

#     forward {
#       target_group {
#         arn    = aws_lb_target_group.aztec-bootstrap-1-target-group.arn
#       }
#     }
#   }
# }

resource "aws_cloudwatch_log_group" "aztec-bootstrap-2-log-group" {
  name              = "/fargate/service/${var.DEPLOY_TAG}/aztec-bootstrap-2"
  retention_in_days = 14
}

resource "aws_service_discovery_service" "aztec-bootstrap-2" {
  name = "${var.DEPLOY_TAG}-aztec-bootstrap-2"

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

resource "aws_ecs_task_definition" "aztec-bootstrap-2" {
  family                   = "${var.DEPLOY_TAG}-aztec-bootstrap-2"
  requires_compatibilities = ["FARGATE"]
  network_mode             = "awsvpc"
  cpu                      = "2048"
  memory                   = "4096"
  execution_role_arn       = data.terraform_remote_state.setup_iac.outputs.ecs_task_execution_role_arn
  task_role_arn            = data.terraform_remote_state.aztec2_iac.outputs.cloudwatch_logging_ecs_role_arn

  container_definitions = <<DEFINITIONS
[
  {
    "name": "${var.DEPLOY_TAG}-aztec-bootstrap-2",
    "image": "${var.ECR_URL}/p2p-bootstrap:aztec3-packages-prod",
    "essential": true,
    "command": ["start"],
    "memoryReservation": 3776,
    "portMappings": [
      {
        "containerPort": ${var.BOOTNODE_2_LISTEN_PORT}
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
        "name": "P2P_TCP_LISTEN_PORT",
        "value": "${var.BOOTNODE_2_LISTEN_PORT}"
      },
      {
        "name": "P2P_TCP_LISTEN_IP",
        "value": "0.0.0.0"
      },
      {
        "name": "PEER_ID_PRIVATE_KEY",
        "value": "${var.BOOTNODE_2_PRIVATE_KEY}"
      },
      {
        "name": "DEBUG",
        "value": "aztec:*"
      }
    ],
    "logConfiguration": {
      "logDriver": "awslogs",
      "options": {
        "awslogs-group": "/fargate/service/${var.DEPLOY_TAG}/aztec-bootstrap-2",
        "awslogs-region": "eu-west-2",
        "awslogs-stream-prefix": "ecs"
      }
    }
  }
]
DEFINITIONS
}

resource "aws_ecs_service" "aztec-bootstrap-2" {
  name                               = "${var.DEPLOY_TAG}-aztec-bootstrap-2"
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
    security_groups = [data.terraform_remote_state.aztec-network_iac.outputs.p2p_security_group_id, data.terraform_remote_state.setup_iac.outputs.security_group_private_id]
  }

  service_registries {
    registry_arn   = aws_service_discovery_service.aztec-bootstrap-2.arn
    container_name = "${var.DEPLOY_TAG}-aztec-bootstrap-2"
    container_port = 80
  }

  load_balancer {
    target_group_arn = aws_lb_target_group.aztec-bootstrap-2-target-group.id
    container_name   = "${var.DEPLOY_TAG}-aztec-bootstrap-2"
    container_port   = var.BOOTNODE_2_LISTEN_PORT
  }

  task_definition = aws_ecs_task_definition.aztec-bootstrap-2.family
}

resource "aws_lb_target_group" "aztec-bootstrap-2-target-group" {
  name        = "aztec-bootstrap-2-target-group"
  port        = var.BOOTNODE_2_LISTEN_PORT
  protocol    = "TCP"
  target_type = "ip"
  vpc_id      = data.terraform_remote_state.setup_iac.outputs.vpc_id

  health_check {
    protocol            = "TCP"
    interval            = 10
    healthy_threshold   = 2
    unhealthy_threshold = 2
    port                = var.BOOTNODE_2_LISTEN_PORT
  }
}

resource "aws_security_group_rule" "allow-bootstrap-2-tcp" {
  type              = "ingress"
  from_port         = var.BOOTNODE_2_LISTEN_PORT
  to_port           = var.BOOTNODE_2_LISTEN_PORT
  protocol          = "tcp"
  cidr_blocks       = ["0.0.0.0/0"]
  security_group_id = data.terraform_remote_state.aztec-network_iac.outputs.p2p_security_group_id
}

## Commented out here and setup manually as terraform (or the aws provider version we are using) has a bug
## NLB listeners can't have a 'weight' property defined. You will see there isn't one here but that doesn't
## stop it trying to automatically specify one and giving an error

# resource "aws_lb_listener" "aztec-bootstrap-2-tcp-listener" {
#   load_balancer_arn = data.terraform_remote_state.aztec-network_iac.outputs.nlb_arn
#   port              = "${var.BOOTNODE_2_LISTEN_PORT}"
#   protocol          = "TCP"

#   tags = {
#     name = "aztec-bootstrap-2-tcp-listener"
#   }

#   default_action {
#     type = "forward"

#     forward {
#       target_group {
#         arn    = aws_lb_target_group.aztec-bootstrap-2-target-group.arn
#       }
#     }
#   }
# }
