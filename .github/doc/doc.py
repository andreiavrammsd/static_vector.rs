#!/usr/bin/env python3

import shutil
import subprocess
import sys
from pathlib import Path


def clean(docs: Path, ghpages: Path):
    shutil.rmtree(docs, ignore_errors=True)
    shutil.rmtree(ghpages, ignore_errors=True)
    docs.mkdir(parents=True)


def main(current_ref: str):
    docs = Path("target/docs")
    build = Path("target/docs_build")
    redirect_template = Path(".github/doc/index_redirect.html")
    template = Path(".github/doc/index_template.html")
    version_template = Path(".github/doc/version_template.html")
    gh_pages_path = Path("target/gh-pages")

    clean(docs, gh_pages_path)

    # Copy existing versions from gh-pages
    cmd(f"git worktree add {gh_pages_path} gh-pages -f")

    for item in gh_pages_path.iterdir():
        if item.is_dir():
            shutil.copytree(item, docs / item.name)

    # Generate docs for current_ref
    print(f"Generating docs for {current_ref}...")
    cmd(f"git checkout {current_ref}")
    cmd(f"cargo doc --no-deps --target-dir={build}")

    (docs / current_ref).mkdir(parents=True, exist_ok=True)
    build_doc = build / "doc"
    shutil.copytree(build_doc, docs / current_ref, dirs_exist_ok=True)
    shutil.copy2(redirect_template, docs / current_ref / "index.html")

    # Load templates
    version_tpl = version_template.read_text()
    template_text = template.read_text()

    # Prepare refs list: master + tags sorted descending semver
    tags = cmd("git tag").splitlines()
    tags.sort(reverse=True, key=lambda s: list(map(int, s.lstrip("v").split("."))))
    refs = ["master"] + tags

    # Generate main index.html
    version_list = ""
    for ver in refs:
        version_list += version_tpl.format(version=ver)

    index_html = template_text.format(versions=version_list)
    (docs / "index.html").write_text(index_html)


def cmd(cmd: str):
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
        main(sys.argv[1] if len(sys.argv) > 1 else "master")
    except subprocess.CalledProcessError as e:
        print(e.stderr)
        sys.exit(1)
