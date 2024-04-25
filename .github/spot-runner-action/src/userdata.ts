import * as github from "@actions/github";
import { ConfigInterface } from "./config";
import { GithubClient } from "./github";

export class UserData {
  config: ConfigInterface;

  constructor(config: ConfigInterface) {
    this.config = config;
  }

  async getUserData(): Promise<string> {
    const ghClient = new GithubClient(this.config);
    const githubActionRunnerVersion = await ghClient.getRunnerVersion();
    // Retrieve runner registration tokens in parallel
    const tokens = await Promise.all(
      Array.from({ length: this.config.githubActionRunnerConcurrency }, () =>
        ghClient.getRunnerRegistrationToken()
      )
    );
    if (!this.config.githubActionRunnerLabel)
      throw Error("failed to object job ID for label");
    const runnerNameBase = `${this.config.githubJobId}-ec2`;
    // space-separated registration tokens
    const tokensSpaceSep = tokens.map((t) => t.token).join(" ");
    const bumpShutdown = `shutdown -c ; shutdown -P +${this.config.ec2InstanceTtl}`;
    // Note, we dont make the runner ephemeral as we start fresh runners as needed
    // and delay shutdowns whenever jobs start
    // TODO could deregister runners right before shutdown starts
    const cmds = [
      "#!/bin/bash",
      `exec 1>/run/log.out 2>&1`, // Log to /run/log.out
      `shutdown -P +${this.config.ec2InstanceTtl}`,
      "cd /run",
      `mkdir -p shutdown-refcount`,
      // Shutdown rules:
      // - github actions job starts and ends always bump +ec2InstanceTtl minutes
      // - when the amount of started jobs (start_run_* files) equal the amount of finished jobs (end_run_* files), we shutdown in 5 minutes
      `echo "${bumpShutdown}; touch /run/shutdown-refcount/start_run_\\$(date +%s)_\\$RANDOM" > /run/delay_shutdown.sh`,
      // `echo "[ \\$(find /run/shutdown-refcount/ -name 'start_run_*' | wc -l) -eq \\$(find /run/shutdown-refcount/ -name 'end_run_*' | wc -l) ] && shutdown -P 5 ; true" > /run/if_refcount0_shutdown.sh`,
      `echo "echo refcounting disabled for now" > /run/if_refcount0_shutdown.sh`,
      `echo "${bumpShutdown}; touch /run/shutdown-refcount/end_run_\\$(date +%s)_\\$RANDOM ; /run/if_refcount0_shutdown.sh " > /run/refcount_and_delay_shutdown.sh`,
      `echo "flock /run/refcount-lock /run/delay_shutdown.sh" > /run/safe_delay_shutdown.sh`,
      `echo "flock /run/refcount-lock /run/refcount_and_delay_shutdown.sh" > /run/safe_refcount_and_delay_shutdown.sh`,
      "chmod +x /run/delay_shutdown.sh",
      "chmod +x /run/refcount_and_delay_shutdown.sh",
      "chmod +x /run/if_refcount0_shutdown.sh",
      "chmod +x /run/safe_refcount_and_delay_shutdown.sh",
      "chmod +x /run/safe_if_refcount0_shutdown.sh",
      "export ACTIONS_RUNNER_HOOK_JOB_STARTED=/run/safe_delay_shutdown.sh",
      "export ACTIONS_RUNNER_HOOK_JOB_COMPLETED=/run/safe_refcount_and_delay_shutdown.sh",
      "mkdir -p actions-runner && cd actions-runner",
      'echo "ACTIONS_RUNNER_HOOK_JOB_STARTED=/run/safe_delay_shutdown.sh" > .env',
      'echo "ACTIONS_RUNNER_HOOK_JOB_COMPLETED=/run/safe_refcount_and_delay_shutdown.sh" > .env',
      `GH_RUNNER_VERSION=${githubActionRunnerVersion}`,
      'case $(uname -m) in aarch64) ARCH="arm64" ;; amd64|x86_64) ARCH="x64" ;; esac && export RUNNER_ARCH=${ARCH}',
      "curl -O -L https://github.com/actions/runner/releases/download/v${GH_RUNNER_VERSION}/actions-runner-linux-${RUNNER_ARCH}-${GH_RUNNER_VERSION}.tar.gz",
      "tar xzf ./actions-runner-linux-${RUNNER_ARCH}-${GH_RUNNER_VERSION}.tar.gz",
      "export RUNNER_ALLOW_RUNASROOT=1",
      "mv externals ..", // we share the big binaries between all the runner folders, symlink instead of copy them
      // Note sharing bin doesn't work due to using it as a folder, and we don't bother splitting up sharing bin
      "rm ./actions-runner-linux-${RUNNER_ARCH}-${GH_RUNNER_VERSION}.tar.gz", // cleanup as we will copy our runner folder
      '[ -n "$(command -v yum)" ] && yum install libicu -y',
      `TOKENS=(${tokensSpaceSep}) ; echo ${tokensSpaceSep} > /run/github-runner-tokens`, // for debugging failed attempts
      `for i in {0..${this.config.githubActionRunnerConcurrency - 1}}; do`,
      `  ( cp -r . ../${runnerNameBase}-$i && ln -s $(pwd)/../externals ../${runnerNameBase}-$i && cd ../${runnerNameBase}-$i && echo \${TOKENS[i]} > .runner-token && ./config.sh --unattended --url https://github.com/${github.context.repo.owner}/${github.context.repo.repo} --token \${TOKENS[i]} --labels ${this.config.githubActionRunnerLabel} --replace --name ${runnerNameBase}-$i ; ./run.sh ) &`,
      "done",
      "wait", // Wait for all background processes to finish
    ];
    console.log(
      "Sending: ",
      cmds.filter((x) => !x.startsWith("TOKENS")).join("\n")
    );
    return Buffer.from(cmds.join("\n")).toString("base64");
  }
}
