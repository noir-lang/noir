
## Aztec packages spot builder

Remember to use npm run build before pushing!

# EC2 Github Action Builder

## TL;DR
Jump to [examples](#example-usage)

## Overview

This is a custom GitHub action to provision and manage self-hosted runners using AWS EC2 On-Demand and/or Spot instances.

It offers multiple spot instance provisioning modes:

- **None:** (default) Strictly On-Demand instances only
- **SpotOnly**: Strictly Spot instances only
- **BestEffort**: Use a Spot instance of same class and size when price is <= On-Demand
  - (Automatic fallback to On-Demand)

Supported operating system AMIs:
- Amazon Linux
- Ubuntu
- Debian

## Why?

### Cost Savings
Operating system	vCPUs	Per-minute rate (USD)
```text
OS     vCPU   GH Price/Minute      EC2 Price/Minute
Linux	2      $0.008             $0.001284 (c5a.large)
Linux	4      $0.016             $0.00257  (c5a.xlarge)
Linux	8      $0.032             $0.00514  (c5a.2xlarge)
Linux	16     $0.064             $0.0114   (c5.4xlarge)
Linux	32     $0.128             $0.02054  (c5a.8xlarge)
Linux	64     $0.256             $0.041067 (c5a.16xlarge)
```

Sources:
- [EC2 On-Demand Pricing](https://aws.amazon.com/ec2/pricing/on-demand/)
- [GH Action Runner Pricing](https://docs.github.com/en/billing/managing-billing-for-github-actions/about-billing-for-github-actions#per-minute-rates)

### Customizable Machine Image
Users can provide their own custom AMI image pre-loaded with all the necessary tooling of their choice saving time and cost.

### Enhance Security
- EC2 instances run within your infrastructure
- Easier to harden runner instances using custom AMIs, Security Groups etc
- Easier monitoring and vulnerability scanning using existing tools (e.g CloudWatch, GuardDuty, AWS Inspector etc)
- Secure networking setup by eliminating any need to expose ports to external service or using Bastion hosts!
- Lower data transfer costs (e.g ECR images, S3 objects etc)

## Setup

### 1. Create GitHub Personal Access Token
1. Create a [fine-grained personal access token](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens#creating-a-fine-grained-personal-access-token)
2. Edit the token permissions and select `Only select repositories` for `Repository access`
3. Select any repositories you wish to use with this action
4. Grant `Read and Write access` for `Administration` access level under Repository permissions
5. Add the token to GitHub Action secrets and note the secret name

### 2. Setup GitHub Secrets for IAM credentials

1. Add your `IAM Access Key ID` and `Secret Access Key` to GitHub Secrets and note the secret names!
2. Modify `${{ secrets.DEPLOY_AWS_ACCESS_KEY_ID }}` and `${{ secrets.DEPLOY_AWS_SECRET_ACCESS_KEY }}` in examples below to match the names of your GH secrets

*Note*: For information about required IAM permissions check **IAM role policy** [here](./docs/CrossAccountIAM.md)

### 3. Collect EC2 information:

- `AWS Region` (e.g `us-west-2`)
- `EC2 AMI ID` for your desired instance type in the region ([Ubuntu AMI Locator](https://cloud-images.ubuntu.com/locator/ec2/))
  - **Important Note:** Only Ubuntu, Amazon Linux and Debian AMIs have been tested
  - To find AMIs **_for other operating systems follow instructions_** [here](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/finding-an-ami.html)
- `EC2 Subnet ID` for any subnet with internet access (Can be private with NAT)
- `EC2 Security Group ID` for any security group which allows all outbound traffic (Default configuration for an empty Security Group)

Note: The security group does not require any in-bound rules. You can add in-bound rules based on your needs (e.g open SSH port 22)

<h2 id="example-usage">
Examples
</h2>

### Standard

- Modify `ec2_spot_instance_strategy` for other deployment strategies. List of all values can be found [here](action.yaml)
- Modify `github_token` value to match the name for your Personal Access Token secret name

```yaml
jobs:
    start-runner:
        timeout-minutes: 5              # normally it only takes 1-2 minutes
        name: Start self-hosted EC2 runner
        runs-on: ubuntu-latest
        permissions:
          actions: write
        steps:
          - name: Start EC2 runner
            id: start-ec2-runner
            uses: NextChapterSoftware/ec2-action-builder@v1
            with:
              github_token: ${{ secrets.GH_PERSONAL_ACCESS_TOKEN }}
              aws_access_key_id: ${{ secrets.DEPLOY_AWS_ACCESS_KEY_ID }}
              aws_secret_access_key: ${{ secrets.DEPLOY_AWS_SECRET_ACCESS_KEY }}
              aws_region: "us-west-2"
              ec2_instance_type: c5.4xlarge
              ec2_ami_id: ami-008fe2fc65df48dac
              ec2_subnet_id: "SUBNET_ID_REDACTED"
              ec2_security_group_id: "SECURITY_GROUP_ID_REDACTED"
              ec2_instance_ttl: 40                # Optional (default is 60 minutes)
              ec2_spot_instance_strategy: None    # Other options are: SpotOnly, BestEffort

    # Job that runs on the self-hosted runner
    run-build:
        timeout-minutes: 1
        needs:
          - start-runner
        runs-on: ${{ github.run_id }}
        steps:
          - run: env
```


### Advanced

- IAM policy and role setup instructions can be found [here](docs/CrossAccountIAM.md)
- Modify `ec2_spot_instance_strategy` for other deployment strategies. List of all values can be found [here](action.yaml)

```yaml
jobs:
    start-runner:
        timeout-minutes: 5                  # normally it only takes 1-2 minutes
        name: Start self-hosted EC2 runner
        runs-on: ubuntu-latest
        permissions:
          actions: write
        steps:
          - name: Start EC2 runner
            id: start-ec2-runner
            uses: NextChapterSoftware/ec2-action-builder@v1
            with:
              aws_access_key_id: ${{ secrets.DEPLOY_AWS_ACCESS_KEY_ID }}
              aws_secret_access_key: ${{ secrets.DEPLOY_AWS_SECRET_ACCESS_KEY }}
              aws_iam_role_arn: "arn:aws:iam::REDACTED:role/REDACTED"
              aws_region: "us-west-2"
              github_token: ${{ secrets.GH_PERSONAL_ACCESS_TOKEN }}
              github_action_runner_version: v2.300.2 # Optional (default is latest release)
              ec2_instance_type: c5.4xlarge
              ec2_ami_id: ami-008fe2fc65df48dac
              ec2_subnet_id: "SUBNET_ID_REDACTED"
              ec2_security_group_id: "SECURITY_GROUP_ID_REDACTED"
              ec2_instance_ttl: 40                          # Optional (default is 60 minutes)
              ec2_spot_instance_strategy: BestEffort    # Other options are: None, BestEffort
              ec2_instance_tags: >                          # Required for IAM role resource permission scoping
                [
                  {"Key": "Owner", "Value": "deploybot"}
                ]

    # Job that runs on the self-hosted runner
    run-build:
        timeout-minutes: 1
        needs:
          - start-runner
        runs-on: ${{ github.run_id }}
        steps:
          - run: env
```
## How it all works under the hood

### General instance launch flow
- Your GitHub personal token is used to obtain a Runner Registration token
- If no explicit runner version has been provided, it will retrieve the latest version number
- It then uses all the provided info to compile an EC2 user-data script which does the following:
  - Set a max TTL on the EC2 instance on startup
  - Create a shutdown script which is executed when jobs end
  - Downloads GitHub Action Runner bundle
  - Unpack Action Runner bundle
  - Configure Runner agent as an **ephemeral** agent
- EC2 instance is launched with the user-data script from previous step
- Once EC2 boot has completed, user-data script is executed
- Runner binary registers itself with GitHub API using the current job ID
- Once the Runner is registered, control is transferred to the next job (this is your build job)
- Upon a job completion (failure/success), Shutdown script is triggered to kill the instance with a 1 minute delay

### Spot instance provisioning
- Script looks up On-Demand price for the supplied instance type
- It will then look up EC2 Spot instance prices using AWS API
- Depending on the mode
  - SpotOnly: It will try to launch a spot instance with On-Demand price as the max price cut-off
  - BestEffort: It will try to launch a spot instance but falls back to On-Demand if prices are too high!

## Other EC2 Considerations
- Each instance is named as "{repo}-{jobID}"
- Default EC2 TTL is 60 minutes
- Other EC2 tags are `github_job_id` and `github_ref`
- Spot instances might be taken away by AWS without any prior notice
