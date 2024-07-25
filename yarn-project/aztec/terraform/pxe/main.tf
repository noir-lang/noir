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

resource "aws_cloudwatch_log_group" "aztec-pxe-log-group" {
  name              = "/fargate/service/${var.DEPLOY_TAG}/aztec-pxe"
  retention_in_days = 14
}

resource "aws_service_discovery_service" "aztec-pxe" {
  name = "${var.DEPLOY_TAG}-aztec-pxe"

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

resource "aws_efs_file_system" "pxe_data_store" {
  creation_token                  = "${var.DEPLOY_TAG}-pxe-data"
  throughput_mode                 = "provisioned"
  provisioned_throughput_in_mibps = 20

  tags = {
    Name = "${var.DEPLOY_TAG}-pxe-data"
  }

  lifecycle_policy {
    transition_to_ia = "AFTER_14_DAYS"
  }
}

resource "aws_efs_mount_target" "public_az1" {
  file_system_id  = aws_efs_file_system.pxe_data_store.id
  subnet_id       = data.terraform_remote_state.setup_iac.outputs.subnet_az1_id
  security_groups = [data.terraform_remote_state.setup_iac.outputs.security_group_public_id]
}

resource "aws_efs_mount_target" "public_az2" {
  file_system_id  = aws_efs_file_system.pxe_data_store.id
  subnet_id       = data.terraform_remote_state.setup_iac.outputs.subnet_az2_id
  security_groups = [data.terraform_remote_state.setup_iac.outputs.security_group_public_id]
}

locals {
  data_dir   = "/usr/src/yarn-project/pxe/data"
  api_prefix = "/${var.DEPLOY_TAG}/aztec-pxe/${var.API_KEY}"
}



resource "aws_ecs_task_definition" "aztec-pxe" {
  family                   = "${var.DEPLOY_TAG}-aztec-pxe"
  network_mode             = "awsvpc"
  cpu                      = 2048
  memory                   = 4096
  requires_compatibilities = ["FARGATE"]
  execution_role_arn       = data.terraform_remote_state.setup_iac.outputs.ecs_task_execution_role_arn
  task_role_arn            = data.terraform_remote_state.aztec2_iac.outputs.cloudwatch_logging_ecs_role_arn

  volume {
    name = "efs-data-store"
    efs_volume_configuration {
      file_system_id = aws_efs_file_system.pxe_data_store.id
    }
  }

  container_definitions = jsonencode([
    {
      name      = "${var.DEPLOY_TAG}-aztec-pxe"
      image     = "${var.DOCKERHUB_ACCOUNT}/aztec:${var.IMAGE_TAG}"
      command   = ["start", "--pxe"]
      essential = true
      portMappings = [
        {
          containerPort = 80
          hostPort      = 80
        }
      ]
      environment = [
        {
          name  = "AZTEC_NODE_URL"
          value = "http://${var.DEPLOY_TAG}-aztec-node-1.local/${var.DEPLOY_TAG}/aztec-node-1/${var.API_KEY}"
        },
        {
          name  = "AZTEC_PORT"
          value = "80"
        },
        {
          name  = "PXE_DATA_DIRECTORY"
          value = local.data_dir
        },
        {
          name  = "API_PREFIX"
          value = local.api_prefix
        }
      ]
      mountPoints = [
        {
          containerPath = local.data_dir
          sourceVolume  = "efs-data-store"
        }
      ]
      logConfiguration = {
        logDriver = "awslogs"
        options = {
          "awslogs-group"         = aws_cloudwatch_log_group.aztec-pxe-log-group.name
          "awslogs-region"        = "eu-west-2"
          "awslogs-stream-prefix" = "ecs"
        }
      }
    }
  ])
}

resource "aws_ecs_service" "aztec-pxe" {
  name                               = "${var.DEPLOY_TAG}-aztec-pxe"
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
    target_group_arn = aws_alb_target_group.pxe_http.arn
    container_name   = "${var.DEPLOY_TAG}-aztec-pxe"
    container_port   = 80
  }

  service_registries {
    registry_arn   = aws_service_discovery_service.aztec-pxe.arn
    container_name = "${var.DEPLOY_TAG}-aztec-pxe"
    container_port = 80
  }

  task_definition = aws_ecs_task_definition.aztec-pxe.family
}

resource "aws_alb_target_group" "pxe_http" {
  name                 = "${var.DEPLOY_TAG}-pxe-http"
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
    name = "${var.DEPLOY_TAG}-pxe-http"
  }
}

resource "aws_lb_listener_rule" "pxe_api" {
  listener_arn = data.terraform_remote_state.aztec2_iac.outputs.alb_listener_arn
  priority     = 400

  action {
    type             = "forward"
    target_group_arn = aws_alb_target_group.pxe_http.arn
  }

  condition {
    path_pattern {
      values = ["${local.api_prefix}*"]
    }
  }
}
