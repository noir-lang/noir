import { ConfigInterface } from "./config";
import { GithubClient } from "./github";

export class UserData {
  config: ConfigInterface;

  constructor(config: ConfigInterface) {
    this.config = config;
  }

  async getUserData(): Promise<string> {
    if (!this.config.githubActionRunnerLabel)
      throw Error("failed to object job ID for label");
    // Note, we dont make the runner ephemeral as we start fresh runners as needed
    // and delay shutdowns whenever jobs start
    const cmds = [
      "#!/bin/bash",
      `exec 1>/run/log.out 2>&1`, // Log to /run/log.out
      "touch /home/ubuntu/.user-data-started",
      `shutdown -P +${this.config.ec2InstanceTtl}`,
      `echo '{"default-address-pools":[{"base":"172.17.0.0/12","size":20}, {"base":"10.99.0.0/12","size":20}, {"base":"192.168.0.0/16","size":24}]}' > /etc/docker/daemon.json`,
      `sudo service docker restart`,
      "sudo wget -q https://github.com/earthly/earthly/releases/download/v0.8.10/earthly-linux-$(dpkg --print-architecture) -O /usr/local/bin/earthly",
      "sudo chmod +x /usr/local/bin/earthly",
      "for i in {1..3} ; do sudo apt install -y brotli && break; sleep 10; done",
      'echo "MaxStartups 1000" >> /etc/ssh/sshd_config',
      'echo "ClientAliveInterval=30" >> /etc/ssh/sshd_config',
      'echo "ClientAliveCountMax=20" >> /etc/ssh/sshd_config',
      "sudo service sshd restart",
      "touch /home/ubuntu/.user-data-finished",
    ];
    console.log(
      "Sending: ",
      cmds.join("\n")
    );
    return Buffer.from(cmds.join("\n")).toString("base64");
  }
}