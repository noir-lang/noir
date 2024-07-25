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

data "terraform_remote_state" "aztec-network_node" {
  backend = "s3"
  config = {
    bucket = "aztec-terraform"
    key    = "${var.DEPLOY_TAG}/aztec-node"
    region = "eu-west-2"
  }
}

locals {
  node_count           = data.terraform_remote_state.aztec-network_node.outputs.node_count
  agents_per_sequencer = var.AGENTS_PER_SEQUENCER
}

resource "aws_cloudwatch_log_group" "aztec-proving-agent-log-group" {
  count             = local.node_count
  name              = "/fargate/service/${var.DEPLOY_TAG}/aztec-proving-agent-group-${count.index + 1}"
  retention_in_days = 14
}

resource "aws_service_discovery_service" "aztec-proving-agent" {
  count = local.node_count
  name  = "${var.DEPLOY_TAG}-aztec-proving-agent-group-${count.index + 1}"

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

# Define task definitions for each node.
resource "aws_ecs_task_definition" "aztec-proving-agent" {
  count                    = local.node_count
  family                   = "${var.DEPLOY_TAG}-aztec-proving-agent-group-${count.index + 1}"
  requires_compatibilities = ["FARGATE"]
  network_mode             = "awsvpc"
  cpu                      = "16384"
  memory                   = "98304"
  execution_role_arn       = data.terraform_remote_state.setup_iac.outputs.ecs_task_execution_role_arn
  task_role_arn            = data.terraform_remote_state.aztec2_iac.outputs.cloudwatch_logging_ecs_role_arn
  container_definitions    = <<DEFINITIONS
[
  {
    "name": "${var.DEPLOY_TAG}-aztec-proving-agent-group-${count.index + 1}",
    "image": "${var.DOCKERHUB_ACCOUNT}/aztec:${var.IMAGE_TAG}",
    "command": ["start", "--prover"],
    "essential": true,
    "memoryReservation": 98304,
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
        "name": "DEBUG",
        "value": "aztec:*"
      },
      {
        "name": "DEPLOY_TAG",
        "value": "${var.DEPLOY_TAG}"
      },
      {
        "name": "AZTEC_NODE_URL",
        "value": "http://${var.DEPLOY_TAG}-aztec-node-${count.index + 1}.local/${var.DEPLOY_TAG}/aztec-node-${count.index + 1}/${var.API_KEY}"
      },
      {
        "name": "PROVER_AGENTS",
        "value": "1"
      },
      {
        "name": "PROVER_REAL_PROOFS",
        "value": "${var.PROVING_ENABLED}"
      }
    ],
    "logConfiguration": {
      "logDriver": "awslogs",
      "options": {
        "awslogs-group": "${aws_cloudwatch_log_group.aztec-proving-agent-log-group[count.index].name}",
        "awslogs-region": "eu-west-2",
        "awslogs-stream-prefix": "ecs"
      }
    }
  }
]
DEFINITIONS
}

resource "aws_ecs_service" "aztec-proving-agent" {
  count                              = local.node_count
  name                               = "${var.DEPLOY_TAG}-aztec-proving-agent-group-${count.index + 1}"
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
    registry_arn   = aws_service_discovery_service.aztec-proving-agent[count.index].arn
    container_name = "${var.DEPLOY_TAG}-aztec-proving-agent-group-${count.index + 1}"
    container_port = 80
  }

  task_definition = aws_ecs_task_definition.aztec-proving-agent[count.index].family
}


# Create CloudWatch metrics for the proving agents
resource "aws_cloudwatch_metric_alarm" "cpu_high" {
  count               = local.node_count
  alarm_name          = "${var.DEPLOY_TAG}-proving-agent-cpu-high-${count.index + 1}"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = "1"
  metric_name         = "CPUUtilization"
  namespace           = "AWS/ECS"
  period              = "60"
  datapoints_to_alarm = 1
  statistic           = "Maximum"
  threshold           = "20"
  alarm_description   = "Alert when CPU utilization is greater than 20%"
  dimensions = {
    ClusterName = data.terraform_remote_state.setup_iac.outputs.ecs_cluster_name
    ServiceName = "${aws_ecs_service.aztec-proving-agent[count.index].name}"
  }
  alarm_actions = [aws_appautoscaling_policy.scale_out[count.index].arn]
}

resource "aws_cloudwatch_metric_alarm" "cpu_low" {
  count               = local.node_count
  alarm_name          = "${var.DEPLOY_TAG}-proving-agent-cpu-low-${count.index + 1}"
  comparison_operator = "LessThanThreshold"
  evaluation_periods  = "3"
  metric_name         = "CPUUtilization"
  namespace           = "AWS/ECS"
  period              = "60"
  datapoints_to_alarm = 3
  statistic           = "Maximum"
  threshold           = "20"
  alarm_description   = "Alarm when CPU utilization is less than 20%"
  dimensions = {
    ClusterName = data.terraform_remote_state.setup_iac.outputs.ecs_cluster_name
    ServiceName = "${aws_ecs_service.aztec-proving-agent[count.index].name}"
  }
  alarm_actions = [aws_appautoscaling_policy.scale_in[count.index].arn]
}

# Create Auto Scaling Target for ECS Service
resource "aws_appautoscaling_target" "ecs_proving_agent" {
  count              = local.node_count
  max_capacity       = local.agents_per_sequencer
  min_capacity       = 1
  resource_id        = "service/${data.terraform_remote_state.setup_iac.outputs.ecs_cluster_id}/${aws_ecs_service.aztec-proving-agent[count.index].name}"
  scalable_dimension = "ecs:service:DesiredCount"
  service_namespace  = "ecs"
}

# Create Scaling Policy for Scaling Out
resource "aws_appautoscaling_policy" "scale_out" {
  count              = local.node_count
  name               = "${var.DEPLOY_TAG}-scale-out-${count.index + 1}"
  policy_type        = "StepScaling"
  resource_id        = aws_appautoscaling_target.ecs_proving_agent[count.index].resource_id
  scalable_dimension = aws_appautoscaling_target.ecs_proving_agent[count.index].scalable_dimension
  service_namespace  = aws_appautoscaling_target.ecs_proving_agent[count.index].service_namespace

  step_scaling_policy_configuration {
    adjustment_type         = "ExactCapacity"
    cooldown                = 60
    metric_aggregation_type = "Maximum"

    step_adjustment {
      scaling_adjustment          = local.agents_per_sequencer
      metric_interval_lower_bound = 0
    }
  }
}

# Create Scaling Policy for Scaling In
resource "aws_appautoscaling_policy" "scale_in" {
  count              = local.node_count
  name               = "${var.DEPLOY_TAG}-scale-in-${count.index + 1}"
  policy_type        = "StepScaling"
  resource_id        = aws_appautoscaling_target.ecs_proving_agent[count.index].resource_id
  scalable_dimension = aws_appautoscaling_target.ecs_proving_agent[count.index].scalable_dimension
  service_namespace  = aws_appautoscaling_target.ecs_proving_agent[count.index].service_namespace

  step_scaling_policy_configuration {
    adjustment_type         = "ExactCapacity"
    cooldown                = 60
    metric_aggregation_type = "Maximum"

    step_adjustment {
      scaling_adjustment          = 1
      metric_interval_upper_bound = 0
    }
  }
}
