#!/usr/bin/env python3

import argparse
import os
import re
import shutil
import subprocess


def get_git_versions():
    result = subprocess.run(["git", "tag"], capture_output=True, text=True, check=True)
    tags = result.stdout.strip().split("\n")
    tags = [tag for tag in tags if tag]

    return sorted(tags, reverse=True)


def render_index(template_path, versions):
    with open(template_path, "r", encoding="utf-8") as file:
        template = file.read()

    versions_html = render_versions(".github/doc/version_template.html", versions)
    index_html = template.format(versions=versions_html)

    return re.sub(r" +", " ", index_html.replace("\n", ""))


def render_versions(template_path, versions):
    with open(template_path, "r", encoding="utf-8") as file:
        template = file.read()

    vars = []
    for version in versions:
        vars.append({
            "version": version,
        })

    return "".join(template.format(**var) for var in vars)


SRC_DIR = ".github/doc"
DOC_DIR = "target/doc"
RENDER_DIR = "target/rendered-doc"
DEV_VERSION = "development"


def generate_version(version):
    subprocess.run(["cargo", "doc", "--no-deps"], check=True)

    version_dst = f"{RENDER_DIR}/{version}"
    shutil.copytree(DOC_DIR, version_dst, dirs_exist_ok=True)


def switch_branch(branch):
    subprocess.run(["git", "checkout", branch], check=True)


def index(versions):
    os.makedirs(RENDER_DIR, exist_ok=True)

    html_output = render_index(f"{SRC_DIR}/index_template.html", versions)

    with open(f"{RENDER_DIR}/index.html", "w", encoding="utf-8") as f:
        f.write(html_output)


def create_version_dirs(version):
    version_dst = f"{RENDER_DIR}/{version}"
    os.makedirs(version_dst, exist_ok=True)
    shutil.copy(f"{SRC_DIR}/index_redirect.html", f"{version_dst}/index.html")


def generate(args):
    versions = get_git_versions()

    for version in versions:
        print(version)

    shutil.rmtree(RENDER_DIR, ignore_errors=True)
    create_version_dirs(DEV_VERSION)
    generate_version(DEV_VERSION)


    for version in versions:
        create_version_dirs(version)

    for version in versions:
        switch_branch(version)
        generate_version(version)

    switch_branch(args.branch)

    index([DEV_VERSION] + versions)


def main():
    parser = argparse.ArgumentParser(description="Documentation")
    subparsers = parser.add_subparsers(dest="command", required=True)

    # `generate` command
    generate_parser = subparsers.add_parser("generate", help="Generate documentation")
    generate_parser.add_argument("--branch", required=True, help="Starting branch")
    generate_parser.set_defaults(func=generate)

    args = parser.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
