#!/usr/bin/env python3
"""
update-version.py

用法:
    python scripts/update-version.py <新版本号>

示例:
    python scripts/update-version.py 0.2.0

功能:
    1. 更新 Cargo.toml 中的 version 字段
    2. 提交改动 (git commit)
    3. 打版本 tag (git tag v<version>)
    4. 推送 commit 和 tag 到远端，触发 GitHub Actions 发布流程
"""

import re
import sys
import subprocess
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
CARGO_TOML = REPO_ROOT / "Cargo.toml"


def validate_semver(version: str) -> None:
    pattern = r"^\d+\.\d+\.\d+(-[a-zA-Z0-9.]+)?(\+[a-zA-Z0-9.]+)?$"
    if not re.fullmatch(pattern, version):
        print("错误：版本号格式无效，需符合 SemVer 规范（如 1.2.3）。")
        sys.exit(1)


def get_current_version() -> str:
    content = CARGO_TOML.read_text(encoding="utf-8")
    match = re.search(r'^version\s*=\s*"([^"]+)"', content, re.MULTILINE)
    if not match:
        print("错误：无法在 Cargo.toml 中找到 version 字段。")
        sys.exit(1)
    return match.group(1)


def update_cargo_toml(new_version: str) -> None:
    content = CARGO_TOML.read_text(encoding="utf-8")
    # 仅替换 [package] 段内的第一个 version 字段，避免误改依赖版本
    new_content, count = re.subn(
        r'^(version\s*=\s*")[^"]+(")',
        rf"\g<1>{new_version}\g<2>",
        content,
        count=1,
        flags=re.MULTILINE,
    )
    if count == 0:
        print("错误：Cargo.toml 中未找到可替换的 version 字段。")
        sys.exit(1)
    CARGO_TOML.write_text(new_content, encoding="utf-8")
    print(f'已更新 Cargo.toml: version = "{new_version}"')


def run(cmd: list[str], **kwargs) -> subprocess.CompletedProcess:
    result = subprocess.run(cmd, cwd=REPO_ROOT, capture_output=True, text=True, **kwargs)
    if result.returncode != 0:
        print(f"命令失败: {' '.join(cmd)}")
        print(result.stderr.strip())
        sys.exit(result.returncode)
    return result


def check_git_clean() -> None:
    result = run(["git", "status", "--porcelain"])
    lines = [l for l in result.stdout.splitlines() if not l.startswith("??")]
    if lines:
        print("错误：工作区存在未提交的变更，请先 commit 或 stash 后再运行此脚本。")
        print("\n".join(lines))
        sys.exit(1)


def tag_exists(tag: str) -> bool:
    result = subprocess.run(["git", "tag", "--list", tag], cwd=REPO_ROOT, capture_output=True, text=True)
    return tag in result.stdout.splitlines()


def main() -> None:
    if len(sys.argv) != 2:
        print(__doc__)
        sys.exit(1)

    new_version = sys.argv[1].lstrip("v")  # 允许用户传入带 v 前缀的版本号
    validate_semver(new_version)

    current_version = get_current_version()
    tag = f"v{new_version}"

    if current_version == new_version:
        print(f"当前版本已是 {new_version}，无需更新。")
        sys.exit(0)

    print(f"当前版本: {current_version}  →  新版本: {new_version}")

    check_git_clean()

    if tag_exists(tag):
        print(f"错误：tag {tag} 已存在，请确认版本号是否重复。")
        sys.exit(1)

    # 1. 更新 Cargo.toml
    update_cargo_toml(new_version)

    # 2. 更新 Cargo.lock（cargo check 会刷新 lock 文件中的版本记录）
    print("正在运行 cargo check 以更新 Cargo.lock ...")
    run(["cargo", "check"])

    # 3. Git commit
    run(["git", "add", "Cargo.toml", "Cargo.lock"])
    run(["git", "commit", "-m", f"chore: bump version to {new_version}"])
    print(f"已提交版本变更。")

    # 4. Git tag
    run(["git", "tag", tag, "-m", f"Release {tag}"])
    print(f"已创建 tag: {tag}")

    # 5. 推送 commit 和 tag
    run(["git", "push"])
    run(["git", "push", "origin", tag])
    print(f"\n✅ 发布流程已触发！tag {tag} 已推送到远端，GitHub Actions 将自动发布到 crates.io。")


if __name__ == "__main__":
    main()
