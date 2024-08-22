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

# Create a fleet.
data "template_file" "user_data" {
  template = <<EOF
#!/bin/bash
echo ECS_CLUSTER=${data.terraform_remote_state.setup_iac.outputs.ecs_cluster_name} >> /etc/ecs/ecs.config
echo 'ECS_INSTANCE_ATTRIBUTES={"group": "${var.DEPLOY_TAG}-bot"}' >> /etc/ecs/ecs.config
EOF
}

resource "aws_launch_template" "bot_launch_template" {
  name                   = "${var.DEPLOY_TAG}-launch-template"
  image_id               = "ami-0cd4858f2b923aa6b"
  instance_type          = "c6a.2xlarge"
  vpc_security_group_ids = [data.terraform_remote_state.setup_iac.outputs.security_group_private_id]

  iam_instance_profile {
    name = data.terraform_remote_state.setup_iac.outputs.ecs_instance_profile_name
  }

  key_name = data.terraform_remote_state.setup_iac.outputs.ecs_instance_key_pair_name

  user_data = base64encode(data.template_file.user_data.rendered)

  tag_specifications {
    resource_type = "instance"
    tags = {
      Name       = "${var.DEPLOY_TAG}-bot"
      prometheus = ""
    }
  }
}

resource "aws_ec2_fleet" "bot_fleet" {
  launch_template_config {
    launch_template_specification {
      launch_template_id = aws_launch_template.bot_launch_template.id
      version            = aws_launch_template.bot_launch_template.latest_version
    }

    override {
      subnet_id         = data.terraform_remote_state.setup_iac.outputs.subnet_az1_private_id
      availability_zone = "eu-west-2a"
      max_price         = "0.15"
    }

    override {
      subnet_id         = data.terraform_remote_state.setup_iac.outputs.subnet_az2_private_id
      availability_zone = "eu-west-2b"
      max_price         = "0.15"
    }
  }

  target_capacity_specification {
    default_target_capacity_type = "on-demand"
    total_target_capacity        = var.BOT_COUNT
    spot_target_capacity         = 0
    on_demand_target_capacity    = var.BOT_COUNT
  }

  terminate_instances                 = true
  terminate_instances_with_expiration = true
}

locals {
  api_prefix = "/${var.DEPLOY_TAG}/aztec-bot/${var.BOT_API_KEY}"
}

resource "aws_ecs_task_definition" "aztec-bot" {
  family                   = "${var.DEPLOY_TAG}-aztec-bot"
  network_mode             = "awsvpc"
  requires_compatibilities = ["EC2"]
  execution_role_arn       = data.terraform_remote_state.setup_iac.outputs.ecs_task_execution_role_arn
  task_role_arn            = data.terraform_remote_state.aztec2_iac.outputs.cloudwatch_logging_ecs_role_arn

  container_definitions = jsonencode([
    {
      name              = "${var.DEPLOY_TAG}-aztec-bot"
      image             = "${var.DOCKERHUB_ACCOUNT}/aztec:${var.DEPLOY_TAG}"
      command           = ["start", "--bot", "--pxe"]
      essential         = true
      cpu               = 8192
      memoryReservation = 14336
      portMappings = [
        {
          containerPort = 80
          hostPort      = 80
        }
      ]
      environment = [
        { name = "BOT_PRIVATE_KEY", value = var.BOT_PRIVATE_KEY },
        { name = "BOT_NO_START", value = var.BOT_NO_START },
        { name = "BOT_TX_INTERVAL_SECONDS", value = var.BOT_TX_INTERVAL_SECONDS },
        { name = "LOG_LEVEL", value = var.LOG_LEVEL },
        { name = "AZTEC_PORT", value = "80" },
        { name = "API_PREFIX", value = local.api_prefix },
        { name = "BOT_PRIVATE_TRANSFERS_PER_TX", value = var.BOT_PRIVATE_TRANSFERS_PER_TX },
        { name = "BOT_PUBLIC_TRANSFERS_PER_TX", value = var.BOT_PUBLIC_TRANSFERS_PER_TX },
        { name = "BOT_TX_MINED_WAIT_SECONDS", value = var.BOT_TX_MINED_WAIT_SECONDS },
        { name = "BOT_FOLLOW_CHAIN", value = var.BOT_FOLLOW_CHAIN },
        { name = "AZTEC_NODE_URL", value = "http://${var.DEPLOY_TAG}-aztec-node-1.local/${var.DEPLOY_TAG}/aztec-node-1/${var.API_KEY}" },
        { name = "PXE_PROVER_ENABLED", value = tostring(var.PROVING_ENABLED) },
        { name = "NETWORK", value = var.DEPLOY_TAG },
        { name = "BOT_FLUSH_SETUP_TRANSACTIONS", value = tostring(var.BOT_FLUSH_SETUP_TRANSACTIONS) },
        { name = "BOT_MAX_PENDING_TXS", value = tostring(var.BOT_MAX_PENDING_TXS) },
        { name = "LOG_JSON", value = "1" }
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
  launch_type                        = "EC2"
  desired_count                      = var.BOT_COUNT
  deployment_maximum_percent         = 100
  deployment_minimum_healthy_percent = 0
  force_new_deployment               = true
  enable_execute_command             = true

  network_configuration {
    subnets = [
      data.terraform_remote_state.setup_iac.outputs.subnet_az1_private_id,
      data.terraform_remote_state.setup_iac.outputs.subnet_az2_private_id
    ]
    security_groups = [data.terraform_remote_state.setup_iac.outputs.security_group_private_id]
  }

  # load_balancer {
  #   target_group_arn = aws_alb_target_group.bot_http.arn
  #   container_name   = "${var.DEPLOY_TAG}-aztec-bot"
  #   container_port   = 80
  # }

  service_registries {
    registry_arn   = aws_service_discovery_service.aztec-bot.arn
    container_name = "${var.DEPLOY_TAG}-aztec-bot"
    container_port = 80
  }

  placement_constraints {
    type       = "memberOf"
    expression = "attribute:group == ${var.DEPLOY_TAG}-bot"
  }

  task_definition = aws_ecs_task_definition.aztec-bot.family
}

# resource "aws_alb_target_group" "bot_http" {
#   name                 = "${var.DEPLOY_TAG}-bot-http"
#   port                 = 80
#   protocol             = "HTTP"
#   target_type          = "ip"
#   vpc_id               = data.terraform_remote_state.setup_iac.outputs.vpc_id
#   deregistration_delay = 5

#   health_check {
#     path                = "${local.api_prefix}/status"
#     matcher             = 200
#     interval            = 10
#     healthy_threshold   = 2
#     unhealthy_threshold = 5
#     timeout             = 5
#   }

#   tags = {
#     name = "${var.DEPLOY_TAG}-bot-http"
#   }
# }

# resource "aws_lb_listener_rule" "bot_api" {
#   listener_arn = data.terraform_remote_state.aztec2_iac.outputs.alb_listener_arn
#   priority     = 700

#   action {
#     type             = "forward"
#     target_group_arn = aws_alb_target_group.bot_http.arn
#   }

#   condition {
#     path_pattern {
#       values = ["${local.api_prefix}*"]
#     }
#   }
# }
