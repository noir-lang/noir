#!/usr/bin/env python3
# ubuntu: apt install python3-blessed
from blessed import Terminal
import os, json, subprocess, sys, time

term = Terminal()
if 'GITHUB_ACTOR' not in os.environ:
    print("Make sure you have GITHUB_ACTOR in your environment variables e.g. .zshrc")
    sys.exit(1)
GITHUB_ACTOR = os.environ['GITHUB_ACTOR']
BRANCH = subprocess.run("git rev-parse --abbrev-ref HEAD", shell=True, text=True, capture_output=True).stdout.strip()

def main():
    selection = -1
    if len(sys.argv) >= 2:
        selection = sys.argv[1]
    else:
        with term.fullscreen(), term.cbreak():
            print(term.home + term.clear)
            while selection not in ('1', '2', '3', '4', '5', 'q'):
                print(term.move_y(1) + "Please select an option:")
                print("1. SSH into build machine")
                print("2. SSH into bench machine")
                print("3. Start/Stop spot machines")
                print("4. Manage Running Jobs")
                print("5. Run ci.yml manually")
                print("q. Quit")
                with term.location(0, term.height - 1):
                    selection = term.inkey()

    if selection == '1':
        ssh_into_machine('x86')
    elif selection == '2':
        ssh_into_machine('bench-x86')
    elif selection == '3':
        manage_spot_instances()
    elif selection == '4':
        manage_ci_workflows()
    elif selection == '5':
        call_ci_workflow()

def ssh_into_machine(suffix):
    ssh_key_path = os.path.expanduser('~/.ssh/build_instance_key')
    if not os.path.exists(ssh_key_path):
        print("SSH key does not exist.")
        return

    # Parse the output to find the public IP address
    for i in range(10):
        # Command to get the instance information
        cmd = f'aws ec2 describe-instances --filters "Name=instance-state-name,Values=running" "Name=tag:Name,Values=aztec-packages-{GITHUB_ACTOR}-{suffix}" --output json --region us-east-2'
        result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
        if result.returncode != 0:
            print("Failed to get AWS instances:", result.stderr)
            return
        try:
            instances_data = json.loads(result.stdout)
            instance = instances_data['Reservations'][0]['Instances'][0]
            instance_ip = instance['PublicIpAddress']
            break
        except (KeyError, IndexError, json.JSONDecodeError) as e:
            print("Error parsing AWS CLI output, trying again:", e)
            if i == 0:
                print("Couldn't find spot, starting spot, and looping until we can find it")
                call_spot_workflow('start')
            elif i == 9:
                print("Couldn't find spot even after creating it!")
                sys.exit(1)
            time.sleep(10)

    # SSH command using the public IP
    ssh_cmd = f"ssh -o StrictHostKeychecking=no -i {ssh_key_path} ubuntu@{instance_ip}"
    print(f"Connecting to {instance_ip}. Consider delaying the impending shutdown and running a process called Runner.Worker to fool the reaper (automation TODO).")
    ssh_process = subprocess.Popen(ssh_cmd, shell=True)
    ssh_process.wait()  # Wait for the SSH session to complete

def call_spot_workflow(action):
    subprocess.run(f'gh workflow run start-spot.yml --ref {BRANCH} --field username="{GITHUB_ACTOR}" --field action="{action}"', shell=True)

def call_ci_workflow():
    print(
        "NOTE: This is mostly useful if impersonating a GITHUB_ACTOR. Usually you rather do Manage Running Jobs and retry."
    )
    subprocess.run(f'gh workflow run ci.yml --ref {BRANCH} --field username="{GITHUB_ACTOR}"', shell=True)

def manage_spot_instances():
    call_spot_workflow(input("Enter one of 'start', 'stop', 'restart':"))

def manage_ci_workflows():
    # Retrieve the most recent workflow run
    cmd = f"gh run list --workflow=ci.yml -u {GITHUB_ACTOR} --limit 5"
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    if result.returncode != 0 or not result.stdout.strip():
        print("Failed to retrieve workflow runs or no runs found.")
        return
    print("Most recent CI run details:")
    print(result.stdout)

    action = input("Enter action 'cancel', 'rerun', 'rerun-all', 'force-cancel' or 'view' (default)") or 'view'
    print(f"\nWill perform {action}")
    run_id = input(f"Enter the run ID to {action}: ")

    if action.lower() == 'cancel':
        subprocess.run(f"gh run cancel {run_id}", shell=True)
    if action.lower() == 'rerun':
        # needed so the spot runners still work
        call_spot_workflow('start')
        subprocess.run(f"gh run rerun {run_id} --failed", shell=True)
    elif action.lower() == 'rerun-all':
        subprocess.run(f"gh run rerun {run_id}", shell=True)
    elif action.lower() == 'force-cancel':
        subprocess.run('gh api --method POST -H "Accept: application/vnd.github+json" -H "X-GitHub-Api-Version: 2022-11-28" ' +
            '/repos/AztecProtocol/aztec-packages/actions/runs/' + run_id + '/force-cancel', shell=True)
    else:
        subprocess.run(f"gh run watch {run_id}", shell=True)

if __name__ == "__main__":
    main()

