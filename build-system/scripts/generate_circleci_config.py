#!/usr/bin/env python3
# Operates on circleci (loaded as json) from stdin
# Outputs filtered circleci without the jobs we don't need to run
# NOTE: This uses the build manifest YAML file to filter the dependency graph in CircleCI BUT it is not one-to-one.
# There is a heuristic here where we expect a job to be associated with a manifest job if it lists the build_manifest.yml job name in its command with a known build command.
import json
import yaml
import re
from concurrent.futures import ProcessPoolExecutor, as_completed
import subprocess
import sys

def eprint(*args, **kwargs):
    print(*args, file=sys.stderr, **kwargs)

# same functionality as query_manifest rebuildPatterns but in bulk
def get_manifest_job_names():
    manifest = yaml.safe_load(open("build_manifest.yml"))
    return list(manifest)

def is_already_built_circleci_job(circleci_job, already_built_manifest_jobs):
    """
    This function checks if a given CircleCI job is associated with a specific already-built manifest job.
    It does so by checking the job's steps for an 'aztec_manifest_key' that contain references to manifest names.
    We want to see at least one such key, and for all such keys to be in 'already_built_manifest_jobs'.
    """
    steps = circleci_job.get("steps", [])
    matching_steps = 0
    for step in steps:
        run_info = step.get("run", "")
        # Check if run_info is a string, short-hand notation
        if isinstance(run_info, str):
            # if there's no run key, or we use string short-hand, continue
            continue
        keys = run_info.get("aztec_manifest_key", [])
        if isinstance(keys, str):
            keys = [keys]
        if not keys: # empty list? continue
            continue
        for key in keys:
            if key not in already_built_manifest_jobs:
                # We have found a different string here - bail out
                return False
        matching_steps += 1
    # All steps have matched - but make sure that's actually more than one step
    return matching_steps > 0

def get_already_built_circleci_job_names(circleci_jobs):
    already_built_manifest_jobs = list(get_already_built_manifest_job_names())
    for job_name, circleci_job in circleci_jobs.items():
        if is_already_built_circleci_job(circleci_job, already_built_manifest_jobs):
            yield job_name

# Helper for multiprocessing
def _get_already_built_manifest_job_names(manifest_name):
    content_hash = subprocess.check_output(['calculate_content_hash', manifest_name]).decode("utf-8")
    completed = subprocess.run(["check_rebuild", f"cache-{content_hash}", manifest_name], stdout=subprocess.DEVNULL)
    if completed.returncode == 0:
        return manifest_name, content_hash
    else:
        return None, None

def get_already_built_manifest_job_names():
    manifest_names = get_manifest_job_names()

    with ProcessPoolExecutor(max_workers=8) as executor:
        futures = {executor.submit(_get_already_built_manifest_job_names, key): key for key in manifest_names}
        for future in as_completed(futures):
            key, content_hash = future.result()
            if key is not None:
                eprint("Detected cached manifest key:", key, "with content hash", content_hash)
                yield key

def remove_jobs_from_workflow(jobs, to_remove):
    """
    Removes jobs from a given CircleCI JSON workflow.

    Parameters:
        jobs (dict): The JSON object representing the CircleCI workflow jobs dependencies portion.
        to_remove (list): The list of jobs to be removed from the workflow.

    Returns:
        dict: The new JSON object with specified jobs removed.
    """

    new_jobs = []
    # Remove specified jobs
    for job in jobs:
        key = next(iter(job))
        if key in to_remove:
            continue
        # remove our filtered jobs from the dependency graph via the requires attribute
        job[key]["requires"] = [r for r in job[key].get("requires", []) if r not in jobs_to_remove]
        new_jobs.append(job)
    return new_jobs

if __name__ == '__main__':
    # The CircleCI workflow as a JSON string (Replace this with your actual workflow)

    # Convert the JSON string to a Python dictionary
    workflow_dict = yaml.safe_load(open('.circleci/config.yml'))

    # # List of jobs to remove
    jobs_to_remove = list(get_already_built_circleci_job_names(workflow_dict["jobs"]))
    for key in jobs_to_remove:
        eprint("Skipping circleci job:", key)

    # Get rid of workflow setup step and setup flag
    workflow_dict["setup"] = False
    del workflow_dict["workflows"]["setup-workflow"]
    # Remove the jobs and get the new workflow
    workflow_dict["workflows"]["system"]["jobs"] = remove_jobs_from_workflow(workflow_dict["workflows"]["system"]["jobs"], jobs_to_remove)
    workflow_dict["workflows"]["system"]["when"] = {"equal":["system","<< pipeline.parameters.workflow >>"]}
    # Convert the new workflow back to JSON string
    new_workflow_json_str = json.dumps(workflow_dict, indent=2)
    print(new_workflow_json_str)