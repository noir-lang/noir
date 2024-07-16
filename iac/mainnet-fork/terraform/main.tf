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

data "terraform_remote_state" "aztec-network_iac" {
  backend = "s3"
  config = {
    bucket = "aztec-terraform"
    key    = "aztec-network/iac"
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


data "aws_alb" "aztec-network_alb" {
  arn = data.terraform_remote_state.aztec2_iac.outputs.alb_arn
}

provider "aws" {
  profile = "default"
  region  = "eu-west-2"
}

resource "aws_service_discovery_service" "aztec_mainnet_fork" {
  name = "${var.DEPLOY_TAG}-mainnet-fork"

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
}

# EFS filesystem for mainnet fork
resource "aws_efs_file_system" "aztec_mainnet_fork_data_store" {
  creation_token = "${var.DEPLOY_TAG}-mainnet-fork-data"

  tags = {
    Name = "${var.DEPLOY_TAG}-mainnet-fork-data"
  }

  lifecycle_policy {
    transition_to_ia = "AFTER_30_DAYS"
  }
}

resource "aws_efs_mount_target" "aztec_fork_private_az1" {
  file_system_id  = aws_efs_file_system.aztec_mainnet_fork_data_store.id
  subnet_id       = data.terraform_remote_state.setup_iac.outputs.subnet_az1_private_id
  security_groups = [data.terraform_remote_state.setup_iac.outputs.security_group_private_id]
}

resource "aws_efs_mount_target" "aztec_fork_private_az2" {
  file_system_id  = aws_efs_file_system.aztec_mainnet_fork_data_store.id
  subnet_id       = data.terraform_remote_state.setup_iac.outputs.subnet_az2_private_id
  security_groups = [data.terraform_remote_state.setup_iac.outputs.security_group_private_id]
}

# Define deployment task and service
resource "aws_ecs_task_definition" "aztec_mainnet_fork" {
  family                   = "${var.DEPLOY_TAG}-mainnet-fork"
  requires_compatibilities = ["FARGATE"]
  network_mode             = "awsvpc"
  cpu                      = "2048"
  memory                   = "4096"
  execution_role_arn       = data.terraform_remote_state.setup_iac.outputs.ecs_task_execution_role_arn

  volume {
    name = "efs-data-store"
    efs_volume_configuration {
      file_system_id = aws_efs_file_system.aztec_mainnet_fork_data_store.id
    }
  }

  container_definitions = jsonencode([
    {
      name      = "${var.DEPLOY_TAG}-mainnet-fork"
      image     = "${var.DOCKERHUB_ACCOUNT}/mainnet-fork:${var.DEPLOY_TAG}"
      essential = true
      environment = [
        {
          name  = "API_KEY"
          value = "${var.API_KEY}"
        },
        {
          name  = "MNEMONIC"
          value = "${var.FORK_MNEMONIC}"
        },
        {
          name  = "INFURA_API_KEY"
          value = "${var.INFURA_API_KEY}"
        },
        {
          name  = "L1_CHAIN_ID"
          value = "${var.L1_CHAIN_ID}"
        },
        {
          name  = "SNAPSHOT_FREQUENCY"
          value = "15"
        }
      ]
      mountPoints = [
        {
          containerPath = "/data"
          sourceVolume  = "efs-data-store"
        }
      ]
      logConfiguration = {
        logDriver = "awslogs"
        options = {
          awslogs-group         = "${aws_cloudwatch_log_group.aztec_mainnet_fork_logs.name}"
          awslogs-region        = "eu-west-2"
          awslogs-stream-prefix = "ecs"
        }
      }
      portMappings = [
        {
          containerPort = 80
          hostPort      = 80
        }
      ]
    }
  ])
}


# ALB to to limit public requests to apikey routes
resource "aws_alb_target_group" "mainnet_fork" {
  name                 = "${var.DEPLOY_TAG}-mainnet-fork"
  port                 = "80"
  protocol             = "HTTP"
  target_type          = "ip"
  vpc_id               = data.terraform_remote_state.setup_iac.outputs.vpc_id
  deregistration_delay = 5
  depends_on = [
    data.aws_alb.aztec-network_alb
  ]

  health_check {
    path                = "/${var.API_KEY}"
    matcher             = "404,400"
    interval            = 300
    healthy_threshold   = 2
    unhealthy_threshold = 10
    timeout             = 120
  }

  tags = {
    name = "${var.DEPLOY_TAG}-mainnet-fork"
  }
}

resource "aws_ecs_service" "aztec_mainnet_fork" {
  name                               = "${var.DEPLOY_TAG}-mainnet-fork"
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
    target_group_arn = aws_alb_target_group.mainnet_fork.arn
    container_name   = "${var.DEPLOY_TAG}-mainnet-fork"
    container_port   = 80
  }

  service_registries {
    registry_arn   = aws_service_discovery_service.aztec_mainnet_fork.arn
    container_name = "${var.DEPLOY_TAG}-mainnet-fork"
    container_port = 80
  }

  task_definition = aws_ecs_task_definition.aztec_mainnet_fork.family
}

resource "aws_cloudwatch_log_group" "aztec_mainnet_fork_logs" {
  name              = "/fargate/services/${var.DEPLOY_TAG}-mainnet_fork"
  retention_in_days = "14"
}

resource "aws_lb_listener_rule" "aztec_mainnet_fork_route" {
  listener_arn = data.terraform_remote_state.aztec2_iac.outputs.mainnet-fork-listener-id

  action {
    type             = "forward"
    target_group_arn = aws_alb_target_group.mainnet_fork.arn
  }

  condition {
    host_header {
      values = ["${var.DEPLOY_TAG}-mainnet-fork.aztec.network"]
    }
  }
}

# mainnet-fork DNS entry.
resource "aws_route53_record" "aztec_mainnet_fork" {
  zone_id = data.terraform_remote_state.aztec2_iac.outputs.aws_route53_zone_id
  name    = "${var.DEPLOY_TAG}-mainnet-fork"
  type    = "A"
  alias {
    name                   = data.aws_alb.aztec-network_alb.dns_name
    zone_id                = data.aws_alb.aztec-network_alb.zone_id
    evaluate_target_health = true
  }
}
