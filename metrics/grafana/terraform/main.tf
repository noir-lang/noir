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

resource "aws_service_discovery_service" "aztec_grafana" {
  name = "aztec-grafana"

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

# Configure an EFS filesystem.
resource "aws_efs_file_system" "aztec_grafana_data_store" {
  creation_token = "aztec-grafana-data-store"

  tags = {
    Name = "aztec-grafana-data-store"
  }

  lifecycle_policy {
    transition_to_ia = "AFTER_14_DAYS"
  }

  lifecycle {
    prevent_destroy = true
  }
}

resource "aws_efs_mount_target" "private_az1" {
  file_system_id  = aws_efs_file_system.aztec_grafana_data_store.id
  subnet_id       = data.terraform_remote_state.setup_iac.outputs.subnet_az1_private_id
  security_groups = [data.terraform_remote_state.setup_iac.outputs.security_group_private_id]
}

resource "aws_efs_mount_target" "private_az2" {
  file_system_id  = aws_efs_file_system.aztec_grafana_data_store.id
  subnet_id       = data.terraform_remote_state.setup_iac.outputs.subnet_az2_private_id
  security_groups = [data.terraform_remote_state.setup_iac.outputs.security_group_private_id]
}

# Define task definition and service.
resource "aws_ecs_task_definition" "aztec_grafana" {
  family                   = "aztec-grafana"
  requires_compatibilities = ["FARGATE"]
  network_mode             = "awsvpc"
  cpu                      = "256"
  memory                   = "512"
  execution_role_arn       = data.terraform_remote_state.setup_iac.outputs.ecs_task_execution_role_arn

  volume {
    name = "aztec-grafana-efs-data-store"
    efs_volume_configuration {
      file_system_id = aws_efs_file_system.aztec_grafana_data_store.id
    }
  }

  container_definitions = <<DEFINITIONS
[
  {
    "name": "aztec-grafana",
    "image": "${var.DOCKERHUB_ACCOUNT}/aztec-grafana:${var.IMAGE_TAG}",
    "essential": true,
    "memoryReservation": 256,
    "portMappings": [
      {
        "containerPort": 80
      }
    ],
    "environment": [
      {
        "name": "GF_AUTH_GITHUB_CLIENT_ID",
        "value": "${var.GRAFANA_CLIENT_ID}"
      },
      {
        "name": "GF_AUTH_GITHUB_CLIENT_SECRET",
        "value": "${var.GRAFANA_CLIENT_SECRET}"
      }
    ],
    "mountPoints": [
      {
        "containerPath": "/var/lib/grafana",
        "sourceVolume": "aztec-grafana-efs-data-store"
      }
    ],
    "logConfiguration": {
      "logDriver": "awslogs",
      "options": {
        "awslogs-group": "/fargate/service/aztec-grafana",
        "awslogs-region": "eu-west-2",
        "awslogs-stream-prefix": "ecs"
      }
    }
  }
]
DEFINITIONS
}

data "aws_ecs_task_definition" "aztec_grafana" {
  task_definition = aws_ecs_task_definition.aztec_grafana.family
}

resource "aws_ecs_service" "aztec_grafana" {
  name                               = "aztec-grafana"
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

  load_balancer {
    target_group_arn = aws_alb_target_group.aztec_grafana.arn
    container_name   = "aztec-grafana"
    container_port   = 80
  }

  service_registries {
    registry_arn = aws_service_discovery_service.aztec_grafana.arn
  }

  task_definition = "${aws_ecs_task_definition.aztec_grafana.family}:${max(aws_ecs_task_definition.aztec_grafana.revision, data.aws_ecs_task_definition.aztec_grafana.revision)}"
}

# Logs
resource "aws_cloudwatch_log_group" "aztec_grafana_logs" {
  name              = "/fargate/service/aztec-grafana"
  retention_in_days = "14"
}

# Configure ALB to route grafana.aztec.network to grafana.
resource "aws_alb_target_group" "aztec_grafana" {
  name                 = "aztec-grafana"
  port                 = "80"
  protocol             = "HTTP"
  target_type          = "ip"
  vpc_id               = data.terraform_remote_state.setup_iac.outputs.vpc_id
  deregistration_delay = 5

  health_check {
    path                = "/"
    matcher             = "302"
    interval            = 10
    healthy_threshold   = 2
    unhealthy_threshold = 5
    timeout             = 3
  }

  tags = {
    name = "aztec-grafana"
  }
}

resource "aws_lb_listener_rule" "api" {
  listener_arn = data.terraform_remote_state.aztec2_iac.outputs.alb_listener_arn
  priority     = 301

  action {
    type             = "forward"
    target_group_arn = aws_alb_target_group.aztec_grafana.arn
  }

  condition {
    host_header {
      values = ["grafana.aztec.network"]
    }
  }
}

data "aws_alb" "aztec2" {
  arn = data.terraform_remote_state.aztec2_iac.outputs.alb_arn
}

# API grafana DNS entry.
resource "aws_route53_record" "metrics" {
  zone_id = data.terraform_remote_state.aztec2_iac.outputs.aws_route53_zone_id
  name    = "grafana"
  type    = "A"
  alias {
    name                   = data.aws_alb.aztec2.dns_name
    zone_id                = data.aws_alb.aztec2.zone_id
    evaluate_target_health = true
  }
}
