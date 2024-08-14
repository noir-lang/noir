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

data "terraform_remote_state" "aztec-network_prover-node" {
  backend = "s3"
  config = {
    bucket = "aztec-terraform"
    key    = "${var.DEPLOY_TAG}/aztec-prover-node"
    region = "eu-west-2"
  }
}

locals {
  node_count        = data.terraform_remote_state.aztec-network_prover-node.outputs.node_count
  agents_per_prover = var.AGENTS_PER_PROVER
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

# Create an autoscaling group for every sequencer. For each group we want 1 permanent on-demand instance to ensure liveness.
# We will fill the remaining desired instances from spot capacity.
data "template_file" "user_data" {
  count    = local.node_count
  template = <<EOF
#!/bin/bash
echo ECS_CLUSTER=${data.terraform_remote_state.setup_iac.outputs.ecs_cluster_name} >> /etc/ecs/ecs.config
echo 'ECS_INSTANCE_ATTRIBUTES={"group": "${var.DEPLOY_TAG}-proving-agent-group-${count.index + 1}"}' >> /etc/ecs/ecs.config
EOF
}

# Launch template for our prover agents
# 32 cores and 128 GB memory
resource "aws_launch_template" "proving-agent-launch-template" {
  count                  = local.node_count
  name                   = "${var.DEPLOY_TAG}-proving-agent-launch-template-${count.index + 1}"
  image_id               = "ami-0cd4858f2b923aa6b"
  instance_type          = "m5.8xlarge"
  vpc_security_group_ids = [data.terraform_remote_state.setup_iac.outputs.security_group_private_id]

  iam_instance_profile {
    name = data.terraform_remote_state.setup_iac.outputs.ecs_instance_profile_name
  }

  key_name = data.terraform_remote_state.setup_iac.outputs.ecs_instance_key_pair_name

  user_data = base64encode(data.template_file.user_data[count.index].rendered)

  tag_specifications {
    resource_type = "instance"
    tags = {
      Name       = "${var.DEPLOY_TAG}-proving-agent-group-${count.index + 1}"
      prometheus = ""
    }
  }
}

resource "aws_ec2_fleet" "aztec_proving_agent_fleet" {
  count = local.node_count
  launch_template_config {
    launch_template_specification {
      launch_template_id = aws_launch_template.proving-agent-launch-template[count.index].id
      version            = aws_launch_template.proving-agent-launch-template[count.index].latest_version
    }

    override {
      subnet_id         = data.terraform_remote_state.setup_iac.outputs.subnet_az1_private_id
      availability_zone = "eu-west-2a"
      max_price         = "0.7"
    }

    override {
      subnet_id         = data.terraform_remote_state.setup_iac.outputs.subnet_az2_private_id
      availability_zone = "eu-west-2b"
      max_price         = "0.7"
    }
  }

  target_capacity_specification {
    default_target_capacity_type = "on-demand"
    total_target_capacity        = local.agents_per_prover
    spot_target_capacity         = 0
    on_demand_target_capacity    = local.agents_per_prover
  }

  terminate_instances                 = true
  terminate_instances_with_expiration = true
}

# Sets up the autoscaling groups
# resource "aws_autoscaling_group" "proving-agent-auto-scaling-group" {
#   count               = local.node_count
#   min_size            = 1
#   max_size            = local.agents_per_prover
#   desired_capacity    = 1
#   vpc_zone_identifier = [data.terraform_remote_state.setup_iac.outputs.subnet_az1_private_id, data.terraform_remote_state.setup_iac.outputs.subnet_az2_private_id]

#   mixed_instances_policy {
#     instances_distribution {
#       on_demand_base_capacity                  = 1
#       on_demand_percentage_above_base_capacity = 100
#       spot_allocation_strategy                 = "lowest-price"
#       spot_max_price                           = "0.7" # Current spot instance price for the m5.8xlarge instance type
#     }

#     launch_template {
#       launch_template_specification {
#         launch_template_id = aws_launch_template.proving-agent-launch-template[count.index].id
#         version            = "$Latest"
#       }
#     }
#   }

#   tag {
#     key                 = "AmazonECSManaged"
#     value               = true
#     propagate_at_launch = true
#   }
# }


# # Capacity provider to manage the scaling of the EC2 instances
# resource "aws_ecs_capacity_provider" "proving-agent-capacity-provider" {
#   count = local.node_count
#   name  = "${var.DEPLOY_TAG}-proving-agent-capacity-provider-${count.index + 1}"


#   auto_scaling_group_provider {
#     auto_scaling_group_arn         = aws_autoscaling_group.proving-agent-auto-scaling-group[count.index].arn
#     managed_termination_protection = "DISABLED"

#     managed_scaling {
#       maximum_scaling_step_size = local.agents_per_prover
#       minimum_scaling_step_size = 1
#       status                    = "ENABLED"
#       target_capacity           = 100
#     }
#   }
# }

# # Update the capacity providers on the cluster
# resource "aws_ecs_cluster_capacity_providers" "proving-agent-capacity-providers" {
#   count        = local.node_count
#   cluster_name = data.terraform_remote_state.setup_iac.outputs.ecs_cluster_name

#   #capacity_providers = [aws_ecs_capacity_provider.proving-agent-capacity-provider[count.index].name]

#   capacity_providers = local.enable_ecs_cluster_auto_scaling == true ? aws_ecs_capacity_provider.asg[*].name : []

#   capacity_providers = (contains(capacity_providers, aws_ecs_capacity_provider.proving-agent-capacity-provider[count.index].name) == false ? concat(capacity_providers, [aws_ecs_capacity_provider.proving-agent-capacity-provider[count.index].name]) : capacity_providers)
# }


# Define task definitions for each node.
resource "aws_ecs_task_definition" "aztec-proving-agent" {
  count                    = local.node_count
  family                   = "${var.DEPLOY_TAG}-aztec-proving-agent-group-${count.index + 1}"
  requires_compatibilities = ["EC2"]
  network_mode             = "awsvpc"
  execution_role_arn       = data.terraform_remote_state.setup_iac.outputs.ecs_task_execution_role_arn
  task_role_arn            = data.terraform_remote_state.aztec2_iac.outputs.cloudwatch_logging_ecs_role_arn
  container_definitions    = <<DEFINITIONS
[
  {
    "name": "${var.DEPLOY_TAG}-aztec-proving-agent-group-${count.index + 1}",
    "image": "${var.DOCKERHUB_ACCOUNT}/aztec:${var.IMAGE_TAG}",
    "command": ["start", "--prover"],
    "essential": true,
    "cpu": 32768,
    "memoryReservation": 122880,
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
        "value": "http://${var.DEPLOY_TAG}-aztec-prover-node-${count.index + 1}.local/${var.DEPLOY_TAG}/aztec-prover-node-${count.index + 1}/${var.API_KEY}"
      },
      {
        "name": "PROVER_AGENT_ENABLED",
        "value": "true"
      },
      {
        "name": "PROVER_AGENT_CONCURRENCY",
        "value": "1"
      },
      {
        "name": "PROVER_REAL_PROOFS",
        "value": "${var.PROVING_ENABLED}"
      },
      {
        "name": "OTEL_EXPORTER_OTLP_ENDPOINT",
        "value": "http://aztec-otel.local:4318"
      },
      {
        "name": "OTEL_SERVICE_NAME",
        "value": "${var.DEPLOY_TAG}-aztec-proving-agent-group-${count.index + 1}"
      },
      {
        "name": "NETWORK_NAME",
        "value": "${var.DEPLOY_TAG}"
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
  launch_type                        = "EC2"
  desired_count                      = local.agents_per_prover
  deployment_maximum_percent         = 100
  deployment_minimum_healthy_percent = 0
  enable_execute_command             = true
  #platform_version                   = "1.4.0"

  # Associate the EC2 capacity provider
  # capacity_provider_strategy {
  #   capacity_provider = "${var.DEPLOY_TAG}-proving-agent-capacity-provider-${count.index + 1}"
  #   weight            = 100
  #   base              = 1
  # }
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

  placement_constraints {
    type       = "memberOf"
    expression = "attribute:group == ${var.DEPLOY_TAG}-proving-agent-group-${count.index + 1}"
  }

  task_definition = aws_ecs_task_definition.aztec-proving-agent[count.index].family
}


# Create CloudWatch metrics for the proving agents
# resource "aws_cloudwatch_metric_alarm" "cpu_high" {
#   count               = local.node_count
#   alarm_name          = "${var.DEPLOY_TAG}-proving-agent-cpu-high-${count.index + 1}"
#   comparison_operator = "GreaterThanThreshold"
#   evaluation_periods  = "1"
#   metric_name         = "CPUUtilization"
#   namespace           = "AWS/ECS"
#   period              = "60"
#   datapoints_to_alarm = 1
#   statistic           = "Maximum"
#   threshold           = "20"
#   alarm_description   = "Alert when CPU utilization is greater than 20%"
#   dimensions = {
#     ClusterName = data.terraform_remote_state.setup_iac.outputs.ecs_cluster_name
#     ServiceName = "${aws_ecs_service.aztec-proving-agent[count.index].name}"
#   }
#   alarm_actions = [aws_appautoscaling_policy.scale_out[count.index].arn]
# }

# resource "aws_cloudwatch_metric_alarm" "cpu_low" {
#   count               = local.node_count
#   alarm_name          = "${var.DEPLOY_TAG}-proving-agent-cpu-low-${count.index + 1}"
#   comparison_operator = "LessThanThreshold"
#   evaluation_periods  = "3"
#   metric_name         = "CPUUtilization"
#   namespace           = "AWS/ECS"
#   period              = "60"
#   datapoints_to_alarm = 3
#   statistic           = "Maximum"
#   threshold           = "20"
#   alarm_description   = "Alarm when CPU utilization is less than 20%"
#   dimensions = {
#     ClusterName = data.terraform_remote_state.setup_iac.outputs.ecs_cluster_name
#     ServiceName = "${aws_ecs_service.aztec-proving-agent[count.index].name}"
#   }
#   alarm_actions = [aws_appautoscaling_policy.scale_in[count.index].arn]
# }

# # Create Auto Scaling Target for ECS Service
# resource "aws_appautoscaling_target" "ecs_proving_agent" {
#   count              = local.node_count
#   max_capacity       = local.agents_per_prover
#   min_capacity       = 1
#   resource_id        = "service/${data.terraform_remote_state.setup_iac.outputs.ecs_cluster_id}/${aws_ecs_service.aztec-proving-agent[count.index].name}"
#   scalable_dimension = "ecs:service:DesiredCount"
#   service_namespace  = "ecs"
# }

# # Create Scaling Policy for Scaling Out
# resource "aws_appautoscaling_policy" "scale_out" {
#   count              = local.node_count
#   name               = "${var.DEPLOY_TAG}-scale-out-${count.index + 1}"
#   policy_type        = "StepScaling"
#   resource_id        = aws_appautoscaling_target.ecs_proving_agent[count.index].resource_id
#   scalable_dimension = aws_appautoscaling_target.ecs_proving_agent[count.index].scalable_dimension
#   service_namespace  = aws_appautoscaling_target.ecs_proving_agent[count.index].service_namespace

#   step_scaling_policy_configuration {
#     adjustment_type         = "ExactCapacity"
#     cooldown                = 60
#     metric_aggregation_type = "Maximum"

#     step_adjustment {
#       scaling_adjustment          = local.agents_per_prover
#       metric_interval_lower_bound = 0
#     }
#   }
# }

# # Create Scaling Policy for Scaling In
# resource "aws_appautoscaling_policy" "scale_in" {
#   count              = local.node_count
#   name               = "${var.DEPLOY_TAG}-scale-in-${count.index + 1}"
#   policy_type        = "StepScaling"
#   resource_id        = aws_appautoscaling_target.ecs_proving_agent[count.index].resource_id
#   scalable_dimension = aws_appautoscaling_target.ecs_proving_agent[count.index].scalable_dimension
#   service_namespace  = aws_appautoscaling_target.ecs_proving_agent[count.index].service_namespace

#   step_scaling_policy_configuration {
#     adjustment_type         = "ExactCapacity"
#     cooldown                = 60
#     metric_aggregation_type = "Maximum"

#     step_adjustment {
#       scaling_adjustment          = 1
#       metric_interval_upper_bound = 0
#     }
#   }
# }
