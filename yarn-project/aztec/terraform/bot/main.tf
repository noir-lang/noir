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

data "terraform_remote_state" "aztec2_iac" {
  backend = "s3"
  config = {
    bucket = "aztec-terraform"
    key    = "aztec2/iac"
    region = "eu-west-2"
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

resource "aws_cloudwatch_log_group" "aztec-bot-log-group" {
  name              = "/fargate/service/${var.DEPLOY_TAG}/aztec-bot"
  retention_in_days = 14
}

resource "aws_service_discovery_service" "aztec-bot" {
  name = "${var.DEPLOY_TAG}-aztec-bot"

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

locals {
  api_prefix = "/${var.DEPLOY_TAG}/aztec-bot/${var.BOT_API_KEY}"
}

resource "aws_ecs_task_definition" "aztec-bot" {
  family                   = "${var.DEPLOY_TAG}-aztec-bot"
  network_mode             = "awsvpc"
  cpu                      = 2048
  memory                   = 4096
  requires_compatibilities = ["FARGATE"]
  execution_role_arn       = data.terraform_remote_state.setup_iac.outputs.ecs_task_execution_role_arn
  task_role_arn            = data.terraform_remote_state.aztec2_iac.outputs.cloudwatch_logging_ecs_role_arn

  container_definitions = jsonencode([
    {
      name      = "${var.DEPLOY_TAG}-aztec-bot"
      image     = "${var.DOCKERHUB_ACCOUNT}/aztec:${var.DEPLOY_TAG}"
      command   = ["start", "--bot"]
      essential = true
      portMappings = [
        {
          containerPort = 80
          hostPort      = 80
        }
      ]
      environment = [
        { name = "BOT_PRIVATE_KEY", value = var.BOT_PRIVATE_KEY },
        { name = "BOT_NO_START", value = "true" },
        { name = "BOT_PXE_URL", value = "http://${var.DEPLOY_TAG}-aztec-pxe-1.local/${var.DEPLOY_TAG}/aztec-pxe-1/${var.API_KEY}" },
        { name = "BOT_TX_INTERVAL_SECONDS", value = 300 },
        { name = "AZTEC_PORT", value = "80" },
        { name = "API_PREFIX", value = local.api_prefix },
      ]
      logConfiguration = {
        logDriver = "awslogs"
        options = {
          "awslogs-group"         = aws_cloudwatch_log_group.aztec-bot-log-group.name
          "awslogs-region"        = "eu-west-2"
          "awslogs-stream-prefix" = "ecs"
        }
      }
    }
  ])
}

resource "aws_ecs_service" "aztec-bot" {
  name                               = "${var.DEPLOY_TAG}-aztec-bot"
  cluster                            = data.terraform_remote_state.setup_iac.outputs.ecs_cluster_id
  launch_type                        = "FARGATE"
  desired_count                      = 1
  deployment_maximum_percent         = 100
  deployment_minimum_healthy_percent = 0
  platform_version                   = "1.4.0"
  force_new_deployment               = true

  network_configuration {
    subnets = [
      data.terraform_remote_state.setup_iac.outputs.subnet_az1_private_id,
      data.terraform_remote_state.setup_iac.outputs.subnet_az2_private_id
    ]
    security_groups = [data.terraform_remote_state.setup_iac.outputs.security_group_private_id]
  }

  load_balancer {
    target_group_arn = aws_alb_target_group.bot_http.arn
    container_name   = "${var.DEPLOY_TAG}-aztec-bot"
    container_port   = 80
  }

  service_registries {
    registry_arn   = aws_service_discovery_service.aztec-bot.arn
    container_name = "${var.DEPLOY_TAG}-aztec-bot"
    container_port = 80
  }

  task_definition = aws_ecs_task_definition.aztec-bot.family
}

resource "aws_alb_target_group" "bot_http" {
  name                 = "${var.DEPLOY_TAG}-bot-http"
  port                 = 80
  protocol             = "HTTP"
  target_type          = "ip"
  vpc_id               = data.terraform_remote_state.setup_iac.outputs.vpc_id
  deregistration_delay = 5

  health_check {
    path                = "${local.api_prefix}/status"
    matcher             = 200
    interval            = 10
    healthy_threshold   = 2
    unhealthy_threshold = 5
    timeout             = 5
  }

  tags = {
    name = "${var.DEPLOY_TAG}-bot-http"
  }
}

resource "aws_lb_listener_rule" "bot_api" {
  listener_arn = data.terraform_remote_state.aztec2_iac.outputs.alb_listener_arn
  priority     = 700

  action {
    type             = "forward"
    target_group_arn = aws_alb_target_group.bot_http.arn
  }

  condition {
    path_pattern {
      values = ["${local.api_prefix}*"]
    }
  }
}
