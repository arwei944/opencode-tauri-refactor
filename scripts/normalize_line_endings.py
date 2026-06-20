"""将仓库内指定文件列表从 CRLF 转换为 LF。

只针对文件列表中的文本文件，跳过二进制和不存在的文件。
"""

import os
import sys

# 需要 LF 规范化的文件（相对仓库根目录）
REPO_ROOT = r"C:\Users\Administrator\opencode-tauri-refactor"

TARGETS = [
    ".github/workflows/ci-test.yml",
    ".github/workflows/tauri-build.yml",
    ".gitignore",
    "CI-CD-GUIDE.md",
    "COMPLETION-REPORT.md",
    "MIGRATION-GUIDE.md",
    "README.md",
    "START-HERE.md",
    "index.html",
    "package.json",
    "src-tauri/Cargo.toml",
    "src-tauri/tauri.conf.json",
    "src/App.tsx",
    "src/hooks/use-electron-api.ts",
    "src/index.css",
    "src/main.tsx",
    "src/types/electron-api.ts",
    "tsconfig.json",
    "vite.config.ts",
]


def to_lf(path: str) -> tuple[int, int]:
    """把单个文件转为 LF。返回 (CRLF->LF 转换行数, 文件大小)。"""
    with open(path, "rb") as f:
        data = f.read()
    before = data.count(b"\r\n")
    if before == 0:
        return (0, len(data))
    new_data = data.replace(b"\r\n", b"\n")
    with open(path, "wb") as f:
        f.write(new_data)
    return (before, len(new_data))


def main() -> int:
    total_files = 0
    total_lines = 0
    for rel in TARGETS:
        full = os.path.join(REPO_ROOT, rel)
        if not os.path.exists(full):
            print(f"  跳过（不存在）: {rel}")
            continue
        n, size = to_lf(full)
        if n > 0:
            total_files += 1
            total_lines += n
            print(f"  {rel}  - {n} 行 CRLF -> LF  ({size} 字节)")
        else:
            print(f"  {rel}  - 已是 LF")
    print(f"\n总计: {total_files} 个文件，{total_lines} 行 CRLF -> LF")
    return 0


if __name__ == "__main__":
    sys.exit(main())
