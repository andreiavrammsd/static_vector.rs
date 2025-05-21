#!/usr/bin/env python3

import subprocess, os, re, shutil


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
        var = {
            "version": version,
        }
        vars.append(var)

    return "".join(template.format(**var) for var in vars)


SRC_DIR = ".github/doc"
DOC_DIR = "target/doc"
RENDER_DIR = "target/doc-rendered"


def generate(args):
    subprocess.run(["cargo", "doc", "--no-deps"], check=True)

    version_dst = f"{RENDER_DIR}/{args.version}"
    os.makedirs(version_dst, exist_ok=True)

    shutil.rmtree(version_dst)
    shutil.copytree(DOC_DIR, version_dst)
    shutil.copy(f"{SRC_DIR}/index_redirect.html", f"{version_dst}/index.html")


def index(args):
    os.makedirs(DOC_DIR, exist_ok=True)

    html_output = render_index(f"{SRC_DIR}/index_template.html", ["latest"] + get_git_versions())

    with open(f"{RENDER_DIR}/index.html", "w", encoding="utf-8") as f:
        f.write(html_output)

import argparse

def main():
    parser = argparse.ArgumentParser(description="Documentation")
    subparsers = parser.add_subparsers(dest="command", required=True)

    # `generate` command
    generate_parser = subparsers.add_parser("generate", help="Generate documentation")
    generate_parser.add_argument("--version", required=True, help="Version to generate")
    generate_parser.set_defaults(func=generate)

    # `index` command
    index_parser = subparsers.add_parser("index", help="Generate index.html")
    index_parser.set_defaults(func=index)

    args = parser.parse_args()
    args.func(args)

if __name__ == "__main__":
    main()
