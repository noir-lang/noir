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

data "terraform_remote_state" "l1_contracts" {
  backend = "s3"
  config = {
    bucket = "aztec-terraform"
    key    = "${var.DEPLOY_TAG}/l1-contracts"
    region = "eu-west-2"
  }
}

# Compute local variables
locals {
  publisher_private_keys = [var.SEQ_1_PUBLISHER_PRIVATE_KEY, var.SEQ_2_PUBLISHER_PRIVATE_KEY]
  node_p2p_private_keys  = [var.NODE_1_PRIVATE_KEY, var.NODE_2_PRIVATE_KEY]
  node_count             = length(local.publisher_private_keys)
  #node_count           = 1
  data_dir             = "/usr/src/yarn-project/aztec/data"
  agents_per_sequencer = var.AGENTS_PER_SEQUENCER
}

resource "aws_cloudwatch_log_group" "aztec-node-log-group" {
  count             = local.node_count
  name              = "/fargate/service/${var.DEPLOY_TAG}/aztec-node-${count.index + 1}"
  retention_in_days = 14
}

resource "aws_service_discovery_service" "aztec-node" {
  count = local.node_count
  name  = "${var.DEPLOY_TAG}-aztec-node-${count.index + 1}"

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

# Configure an EFS filesystem.
resource "aws_efs_file_system" "node_data_store" {
  count                           = local.node_count
  creation_token                  = "${var.DEPLOY_TAG}-node-${count.index + 1}-data"
  throughput_mode                 = "provisioned"
  provisioned_throughput_in_mibps = 20

  tags = {
    Name = "${var.DEPLOY_TAG}-node-${count.index + 1}-data"
  }

  lifecycle_policy {
    transition_to_ia = "AFTER_14_DAYS"
  }
}

# resource "aws_efs_mount_target" "private_az1" {
#   count           = local.node_count
#   file_system_id  = aws_efs_file_system.node_data_store[count.index].id
#   subnet_id       = data.terraform_remote_state.setup_iac.outputs.subnet_az1_private_id
#   security_groups = [data.terraform_remote_state.setup_iac.outputs.security_group_private_id]
# }

# resource "aws_efs_mount_target" "private_az2" {
#   count           = local.node_count
#   file_system_id  = aws_efs_file_system.node_data_store[count.index].id
#   subnet_id       = data.terraform_remote_state.setup_iac.outputs.subnet_az2_private_id
#   security_groups = [data.terraform_remote_state.setup_iac.outputs.security_group_private_id]
# }

resource "aws_efs_mount_target" "public_az1" {
  count           = local.node_count
  file_system_id  = aws_efs_file_system.node_data_store[count.index].id
  subnet_id       = data.terraform_remote_state.setup_iac.outputs.subnet_az1_id
  security_groups = [data.terraform_remote_state.setup_iac.outputs.security_group_public_id]
}

resource "aws_efs_mount_target" "public_az2" {
  count           = local.node_count
  file_system_id  = aws_efs_file_system.node_data_store[count.index].id
  subnet_id       = data.terraform_remote_state.setup_iac.outputs.subnet_az2_id
  security_groups = [data.terraform_remote_state.setup_iac.outputs.security_group_public_id]
}

# Define task definitions for each node.
resource "aws_ecs_task_definition" "aztec-node" {
  count                    = local.node_count
  family                   = "${var.DEPLOY_TAG}-aztec-node-${count.index + 1}"
  requires_compatibilities = ["FARGATE"]
  network_mode             = "awsvpc"
  cpu                      = "2048"
  memory                   = "4096"
  execution_role_arn       = data.terraform_remote_state.setup_iac.outputs.ecs_task_execution_role_arn
  task_role_arn            = data.terraform_remote_state.aztec2_iac.outputs.cloudwatch_logging_ecs_role_arn

  volume {
    name = "efs-data-store"
    efs_volume_configuration {
      file_system_id = aws_efs_file_system.node_data_store[count.index].id
    }
  }

  container_definitions = <<DEFINITIONS
[
  {
    "name": "${var.DEPLOY_TAG}-aztec-node-${count.index + 1}",
    "image": "${var.FULL_IMAGE}",
    "command": ["start", "--node", "--archiver", "--sequencer", "--prover"],
    "essential": true,
    "memoryReservation": 3776,
    "portMappings": [
      {
        "containerPort": 80
      },
      {
        "containerPort": ${var.NODE_P2P_TCP_PORT + count.index},
        "protocol": "tcp"
      },
      {
        "containerPort": ${var.NODE_P2P_UDP_PORT + count.index},
        "protocol": "udp"
      }
    ],
    "environment": [
      {
        "name": "NODE_ENV",
        "value": "production"
      },
      {
        "name": "DEPLOY_TAG",
        "value": "${var.DEPLOY_TAG}"
      },
      {
        "name": "DEPLOY_AZTEC_CONTRACTS",
        "value": "false"
      },
      {
        "name": "AZTEC_PORT",
        "value": "80"
      },
      {
        "name": "DEBUG",
        "value": "aztec:*,-json-rpc:json_proxy:*,-aztec:avm_simulator:*"
      },
      {
        "name": "ETHEREUM_HOST",
        "value": "https://${var.DEPLOY_TAG}-mainnet-fork.aztec.network:8545/${var.API_KEY}"
      },
      {
        "name": "DATA_DIRECTORY",
        "value": "${local.data_dir}"
      },
      {
        "name": "ARCHIVER_POLLING_INTERVAL",
        "value": "10000"
      },
      {
        "name": "SEQ_RETRY_INTERVAL",
        "value": "10000"
      },
      {
        "name": "SEQ_MAX_TX_PER_BLOCK",
        "value": "${var.SEQ_MAX_TX_PER_BLOCK}"
      },
      {
        "name": "SEQ_MIN_TX_PER_BLOCK",
        "value": "${var.SEQ_MIN_TX_PER_BLOCK}"
      },
      {
        "name": "SEQ_PUBLISHER_PRIVATE_KEY",
        "value": "${local.publisher_private_keys[count.index]}"
      },

      {
        "name": "ROLLUP_CONTRACT_ADDRESS",
        "value": "${var.ROLLUP_CONTRACT_ADDRESS}"
      },
      {
        "name": "INBOX_CONTRACT_ADDRESS",
        "value": "${var.INBOX_CONTRACT_ADDRESS}"
      },
      {
        "name": "OUTBOX_CONTRACT_ADDRESS",
        "value": "${var.OUTBOX_CONTRACT_ADDRESS}"
      },
      {
        "name": "REGISTRY_CONTRACT_ADDRESS",
        "value": "${var.REGISTRY_CONTRACT_ADDRESS}"
      },
      {
        "name": "AVAILABILITY_ORACLE_CONTRACT_ADDRESS",
        "value": "${var.AVAILABILITY_ORACLE_CONTRACT_ADDRESS}"
      },
      {
        "name": "GAS_TOKEN_CONTRACT_ADDRESS",
        "value": "${var.GAS_TOKEN_CONTRACT_ADDRESS}"
      },
      {
        "name": "GAS_PORTAL_CONTRACT_ADDRESS",
        "value": "${var.GAS_PORTAL_CONTRACT_ADDRESS}"
      },
      {
        "name": "API_KEY",
        "value": "${var.API_KEY}"
      },
      {
        "name": "API_PREFIX",
        "value": "/${var.DEPLOY_TAG}/aztec-node-${count.index + 1}"
      },
      {
        "name": "P2P_TCP_LISTEN_ADDR",
        "value": "0.0.0.0:${var.NODE_P2P_TCP_PORT + count.index}"
      },
      {
        "name": "P2P_UDP_LISTEN_ADDR",
        "value": "0.0.0.0:${var.NODE_P2P_UDP_PORT + count.index}"
      },
      {
        "name": "P2P_TCP_ANNOUNCE_ADDR",
        "value": ":${var.NODE_P2P_TCP_PORT + count.index}"
      },
      {
        "name": "P2P_UDP_ANNOUNCE_ADDR",
        "value": ":${var.NODE_P2P_UDP_PORT + count.index}"
      },
      {
        "name": "P2P_QUERY_FOR_IP",
        "value": "true"
      },
      {
        "name": "BOOTSTRAP_NODES",
        "value": "enr:-JO4QNvVz7yYHQ4nzZQ7JCng9LOQkDnFqeLntDEfrAAGOS_eMFWOE4ZlyjYKb3J-yCGu8xoXXEUnUqI8iTJj1K43KH0EjWF6dGVjX25ldHdvcmsBgmlkgnY0gmlwhA0pYm6Jc2VjcDI1NmsxoQLzGvsxdzM9VhPjrMnxLmMxvrEcvSg-QZq7PWXDnnIy1YN1ZHCCnjQ"
      },
      {
        "name": "P2P_ENABLED",
        "value": "${var.P2P_ENABLED}"
      },
      {
        "name": "CHAIN_ID",
        "value": "${var.CHAIN_ID}"
      },
      {
        "name": "PEER_ID_PRIVATE_KEY",
        "value": "${local.node_p2p_private_keys[count.index]}"
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
        "name": "P2P_BLOCK_CHECK_INTERVAL_MS",
        "value": "1000"
      },
      {
        "name": "P2P_PEER_CHECK_INTERVAL_MS",
        "value": "2000"
      },
      {
        "name": "PROVER_AGENTS",
        "value": "0"
      },
      {
        "name": "PROVER_REAL_PROOFS",
        "value": "${var.PROVING_ENABLED}"
      }
    ],
    "mountPoints": [
      {
        "containerPath": "${local.data_dir}",
        "sourceVolume": "efs-data-store"
      }
    ],
    "logConfiguration": {
      "logDriver": "awslogs",
      "options": {
        "awslogs-group": "/fargate/service/${var.DEPLOY_TAG}/aztec-node-${count.index + 1}",
        "awslogs-region": "eu-west-2",
        "awslogs-stream-prefix": "ecs"
      }
    }
  }
]
DEFINITIONS
}

resource "aws_ecs_service" "aztec-node" {
  count                              = local.node_count
  name                               = "${var.DEPLOY_TAG}-aztec-node-${count.index + 1}"
  cluster                            = data.terraform_remote_state.setup_iac.outputs.ecs_cluster_id
  launch_type                        = "FARGATE"
  desired_count                      = 1
  deployment_maximum_percent         = 100
  deployment_minimum_healthy_percent = 0
  platform_version                   = "1.4.0"


  network_configuration {
    assign_public_ip = true
    subnets = [
      data.terraform_remote_state.setup_iac.outputs.subnet_az1_id
    ]
    security_groups = [data.terraform_remote_state.aztec-network_iac.outputs.p2p_security_group_id, data.terraform_remote_state.setup_iac.outputs.security_group_private_id]
  }

  load_balancer {
    target_group_arn = aws_alb_target_group.aztec-node-http[count.index].arn
    container_name   = "${var.DEPLOY_TAG}-aztec-node-${count.index + 1}"
    container_port   = 80
  }


  # load_balancer {
  #   target_group_arn = aws_lb_target_group.aztec-node-tcp[count.index].arn
  #   container_name   = "${var.DEPLOY_TAG}-aztec-node-${count.index + 1}"
  #   container_port   = var.NODE_P2P_TCP_PORT + count.index
  # }

  # load_balancer {
  #   target_group_arn = aws_lb_target_group.aztec-node-udp[count.index].arn
  #   container_name   = "${var.DEPLOY_TAG}-aztec-node-${count.index + 1}"
  #   container_port   = var.NODE_P2P_UDP_PORT + count.index
  # }

  service_registries {
    registry_arn   = aws_service_discovery_service.aztec-node[count.index].arn
    container_name = "${var.DEPLOY_TAG}-aztec-node-${count.index + 1}"
    container_port = 80
  }

  task_definition = aws_ecs_task_definition.aztec-node[count.index].family
}

# Configure ALB to route /aztec-node to server.
resource "aws_alb_target_group" "aztec-node-http" {
  count                = local.node_count
  name                 = "${var.DEPLOY_TAG}-node-${count.index + 1}-http-target"
  port                 = 80
  protocol             = "HTTP"
  target_type          = "ip"
  vpc_id               = data.terraform_remote_state.setup_iac.outputs.vpc_id
  deregistration_delay = 5

  health_check {
    path                = "/${var.DEPLOY_TAG}/aztec-node-${count.index + 1}/status"
    matcher             = "200"
    interval            = 10
    healthy_threshold   = 2
    unhealthy_threshold = 5
    timeout             = 5
  }

  tags = {
    name = "${var.DEPLOY_TAG}-aztec-node-${count.index + 1}"
  }
}

resource "aws_lb_listener_rule" "api" {
  count        = local.node_count
  listener_arn = data.terraform_remote_state.aztec2_iac.outputs.alb_listener_arn
  priority     = 500 + count.index

  action {
    type             = "forward"
    target_group_arn = aws_alb_target_group.aztec-node-http[count.index].arn
  }

  condition {
    path_pattern {
      values = ["/${var.DEPLOY_TAG}/aztec-node-${count.index + 1}*"]
    }
  }
}

# resource "aws_lb_target_group" "aztec-node-tcp" {
#   count       = local.node_count
#   name        = "${var.DEPLOY_TAG}-node-${count.index + 1}-p2p-tcp-target"
#   port        = var.NODE_P2P_TCP_PORT + count.index
#   protocol    = "TCP"
#   target_type = "ip"
#   vpc_id      = data.terraform_remote_state.setup_iac.outputs.vpc_id

#   health_check {
#     protocol            = "TCP"
#     interval            = 10
#     healthy_threshold   = 2
#     unhealthy_threshold = 2
#     port                = var.NODE_P2P_TCP_PORT + count.index
#   }
# }

resource "aws_security_group_rule" "allow-node-tcp-in" {
  count             = local.node_count
  type              = "ingress"
  from_port         = var.NODE_P2P_TCP_PORT + count.index
  to_port           = var.NODE_P2P_TCP_PORT + count.index
  protocol          = "tcp"
  cidr_blocks       = ["0.0.0.0/0"]
  security_group_id = data.terraform_remote_state.aztec-network_iac.outputs.p2p_security_group_id
}

resource "aws_security_group_rule" "allow-node-tcp-out" {
  count             = local.node_count
  type              = "egress"
  from_port         = var.NODE_P2P_TCP_PORT + count.index
  to_port           = var.NODE_P2P_TCP_PORT + count.index
  protocol          = "tcp"
  cidr_blocks       = ["0.0.0.0/0"]
  security_group_id = data.terraform_remote_state.aztec-network_iac.outputs.p2p_security_group_id
}

# resource "aws_lb_listener" "aztec-node-tcp-listener" {
#   count             = local.node_count
#   load_balancer_arn = data.terraform_remote_state.aztec-network_iac.outputs.nlb_arn
#   port              = var.NODE_P2P_TCP_PORT + count.index
#   protocol          = "TCP"

#   tags = {
#     name = "aztec-node-${count.index}-tcp-listener"
#   }

#   default_action {
#     type             = "forward"
#     target_group_arn = aws_lb_target_group.aztec-node-tcp[count.index].arn
#   }
# }


# resource "aws_lb_target_group" "aztec-node-udp" {
#   count       = local.node_count
#   name        = "${var.DEPLOY_TAG}-node-${count.index + 1}-p2p-udp-target"
#   port        = var.NODE_P2P_UDP_PORT + count.index
#   protocol    = "UDP"
#   target_type = "ip"
#   vpc_id      = data.terraform_remote_state.setup_iac.outputs.vpc_id

#   health_check {
#     protocol            = "TCP"
#     interval            = 10
#     healthy_threshold   = 2
#     unhealthy_threshold = 2
#     port                = var.NODE_P2P_TCP_PORT + count.index
#   }
# }

resource "aws_security_group_rule" "allow-node-udp-in" {
  type              = "ingress"
  from_port         = var.NODE_P2P_UDP_PORT
  to_port           = var.NODE_P2P_UDP_PORT + 100
  protocol          = "udp"
  cidr_blocks       = ["0.0.0.0/0"]
  security_group_id = data.terraform_remote_state.aztec-network_iac.outputs.p2p_security_group_id
}

resource "aws_security_group_rule" "allow-node-udp-out" {
  type              = "egress"
  from_port         = var.NODE_P2P_UDP_PORT
  to_port           = var.NODE_P2P_UDP_PORT + 100
  protocol          = "udp"
  cidr_blocks       = ["0.0.0.0/0"]
  security_group_id = data.terraform_remote_state.aztec-network_iac.outputs.p2p_security_group_id
}

# resource "aws_lb_listener" "aztec-node-udp-listener" {
#   count             = local.node_count
#   load_balancer_arn = data.terraform_remote_state.aztec-network_iac.outputs.nlb_arn
#   port              = var.NODE_P2P_UDP_PORT + count.index
#   protocol          = "UDP"

#   tags = {
#     name = "aztec-node-${count.index}-udp-listener"
#   }

#   default_action {
#     type             = "forward"
#     target_group_arn = aws_lb_target_group.aztec-node-udp[count.index].arn
#   }
# }



// Configuration for proving agents

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
    command = "${path.module}/servicediscovery-drain.sh ${self.id}"
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
    "image": "${var.FULL_IMAGE}",
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
        "value": "http://${var.DEPLOY_TAG}-aztec-node-${count.index + 1}.local/${var.DEPLOY_TAG}/aztec-node-${count.index + 1}"
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
