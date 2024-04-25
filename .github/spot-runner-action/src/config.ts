import * as core from "@actions/core";
import * as github from "@actions/github";

export interface ConfigInterface {
  awsAccessKeyId: string;
  awsSecretAccessKey: string;
  awsRegion: string;
  awsIamRoleArn: string;
  awsAssumeRole: boolean;

  githubToken: string;
  githubJobId: string;
  githubRef: string;
  githubRepo: string;
  githubActionRunnerVersion: string;
  githubActionRunnerConcurrency: number;
  githubActionRunnerLabel: string;
  subaction: string;

  ec2InstanceType: string[];
  ec2AmiId: string;
  ec2InstanceIamRole: string;
  ec2InstanceTags: string;
  ec2InstanceTtl: string;
  ec2SecurityGroupId: string;
  ec2SubnetId: string;
  ec2KeyName: string;
  ec2SpotInstanceStrategy: string;
}

export class ActionConfig implements ConfigInterface {
  awsAccessKeyId: string;
  awsSecretAccessKey: string;
  awsRegion: string;
  awsIamRoleArn: string;
  awsAssumeRole: boolean;

  githubToken: string;
  githubJobId: string;
  githubRef: string;
  githubRepo: string;
  githubActionRunnerVersion: string;
  githubActionRunnerLabel: string;
  githubActionRunnerConcurrency: number;
  subaction: string;

  ec2InstanceType: string[];
  ec2AmiId: string;
  ec2InstanceIamRole: string;
  ec2InstanceTags: string;
  ec2InstanceTtl: string;
  ec2SecurityGroupId: string;
  ec2SubnetId: string;
  ec2KeyName: string;
  ec2SpotInstanceStrategy: string;

  constructor() {
    // AWS account and credentials params
    this.awsAccessKeyId = core.getInput("aws_access_key_id");
    this.awsSecretAccessKey = core.getInput("aws_secret_access_key");
    this.awsRegion = core.getInput("aws_region");
    this.awsIamRoleArn = core.getInput("aws_iam_role_arn");
    this.awsAssumeRole = this.awsIamRoleArn ? true : false;

    // Github params
    this.githubToken = core.getInput("github_token");
    this.githubJobId = core.getInput("runner_label");
    this.githubRef = github.context.ref;
    this.githubRepo = github.context.repo.repo;
    this.githubActionRunnerVersion = core.getInput(
      "github_action_runner_version"
    );
    this.githubActionRunnerLabel = this.githubJobId;
    this.subaction = core.getInput("subaction");
    this.githubActionRunnerConcurrency = +core.getInput("runner_concurrency");

    // Ec2 params
    this.ec2InstanceType = core.getInput("ec2_instance_type").split(" ");
    this.ec2AmiId = core.getInput("ec2_ami_id");
    this.ec2InstanceIamRole = core.getInput("ec2_instance_iam_role");
    this.ec2InstanceTags = core.getInput("ec2_instance_tags");
    this.ec2InstanceTtl = core.getInput("ec2_instance_ttl");
    this.ec2SubnetId = core.getInput("ec2_subnet_id");
    this.ec2KeyName = core.getInput("ec2_key_name");
    this.ec2SecurityGroupId = core.getInput("ec2_security_group_id");
    this.ec2SpotInstanceStrategy = core
      .getInput("ec2_spot_instance_strategy")
      .toLowerCase();
  }
}
