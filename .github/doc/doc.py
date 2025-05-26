#!/usr/bin/env python3

"""
Multi-version Rust documentation for GitHub Pages.

- Generates documentation for default branch and for the given Git ref.
- Generates index.html, for each ref, to redirect to crate documentation.
- Generates main index.html that lists all versions (only semver Git tag) + default branch (skips anything else).

Usage:
    ./doc.py [ref]

If no ref is provided, it defaults to using "master".
"""

import re
import shutil
import subprocess
import sys
from pathlib import Path


def clean(docs: Path, gh_pages: Path):
    """Remove and recreate documentation directories."""
    shutil.rmtree(docs, ignore_errors=True)
    shutil.rmtree(gh_pages, ignore_errors=True)
    docs.mkdir(parents=True)


def copy_existing_versions(gh_pages: Path, docs: Path):
    """Copy existing versioned documentation from gh-pages to docs."""
    run_cmd(f"git worktree add {gh_pages} gh-pages -f")

    for item in gh_pages.iterdir():
        if item.is_dir():
            shutil.copytree(item, docs / item.name)


def generate_docs(build: Path, docs: Path, git_ref: str, redirect_template: Path):
    """Generate documentation for the given Git reference."""
    print(f"Generating docs for {git_ref}...")
    run_cmd(f"git checkout {git_ref} --")
    run_cmd(f"cargo doc --no-deps --target-dir={build}")

    (docs / git_ref).mkdir(parents=True, exist_ok=True)
    build_doc = build / "doc"
    shutil.copytree(build_doc, docs / git_ref, dirs_exist_ok=True)
    shutil.copy2(redirect_template, docs / git_ref / "index.html")


def get_version_refs() -> list[str]:
    """Get a list of version refs: sorted tags."""
    tags = run_cmd("git tag").splitlines()
    tags.sort(reverse=True, key=lambda s: list(map(int, s.lstrip("v").split("."))))
    return tags


def generate_main_index(
    docs: Path, version_template: Path, index_template: Path, refs: list[str]
):
    """Generate the main index.html for the docs site."""
    version_tpl = version_template.read_text()
    index_tpl = index_template.read_text()

    version_list = ""
    for ver in refs:
        version_list += version_tpl.format(version=ver)

    index_html = index_tpl.format(versions=version_list)
    index_html = re.sub(r" +", " ", index_html.replace("\n", ""))
    (docs / "index.html").write_text(index_html)


def main(ref: str, default_ref: str):
    docs = Path("target/docs")
    build = Path("target/docs_build")
    redirect_template = Path(".github/doc/index_redirect.html")
    index_template = Path(".github/doc/index_template.html")
    version_template = Path(".github/doc/version_template.html")
    gh_pages = Path("target/gh-pages")

    clean(docs, gh_pages)
    copy_existing_versions(gh_pages, docs)
    generate_docs(build, docs, ref, redirect_template)

    refs = [default_ref] + get_version_refs()
    generate_main_index(docs, version_template, index_template, refs)


def run_cmd(cmd: str):
    result = subprocess.run(
        cmd.split(),
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=True,
        text=True,
    )
    return result.stdout


if __name__ == "__main__":
    try:
        default_branch = "master"
        main(sys.argv[1] if len(sys.argv) > 1 else default_branch, default_branch)
    except subprocess.CalledProcessError as e:
        print(e.stderr)
        sys.exit(1)
