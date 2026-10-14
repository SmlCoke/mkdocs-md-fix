"""Package and smoke-test a native release build using only the Python stdlib."""

from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tempfile
import tomllib
import zipfile


def verify(package: Path, suffix: str, version: str) -> None:
    """Exercise the extracted executables, not just files in the build directory."""
    mmf = package / f"mmf{suffix}"
    shell = package / f"mmf_shell{suffix}"
    help_result = subprocess.run([str(mmf), "help"], capture_output=True, text=True, check=True)
    if "mmf check <path>... [-r]" not in help_result.stdout:
        raise RuntimeError("missing command-line help")
    shell_result = subprocess.run(
        [str(shell)], input="help\nexit\n", capture_output=True, text=True, check=True
    )
    if f"Version {version}" not in shell_result.stdout:
        raise RuntimeError("interactive banner does not match Cargo.toml")

    with tempfile.TemporaryDirectory(prefix="mmf-smoke-") as sample_dir:
        sample = Path(sample_dir) / "sample.md"
        original = "Intro:\n- item\n  - child\nEnd\n\nCXL.mem  允许主机处理器使用内 存。`中  文` $a  + b$\n"
        expected = "Intro:\n\n- item\n    - child\n\nEnd\n\nCXL.mem 允许主机处理器使用内存。`中  文` $a  + b$\n"
        sample.write_bytes(original.encode("utf-8"))
        checked = subprocess.run([str(mmf), "check", str(sample)], capture_output=True)
        if checked.returncode != 1:
            raise RuntimeError("check should report formatting issues with exit code 1")
        preview = subprocess.run([str(mmf), "fix", str(sample)], capture_output=True, check=True)
        if b"diff -- mmf" not in preview.stdout or sample.read_bytes() != original.encode():
            raise RuntimeError("fix must preview a diff without changing the file")
        subprocess.run([str(mmf), "write", str(sample)], capture_output=True, check=True)
        if sample.read_bytes() != expected.encode():
            raise RuntimeError("write did not apply list/prose rules or preserve code/math")
        subprocess.run([str(mmf), "check", str(sample)], capture_output=True, check=True)


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: package_release.py <target-triple>")
    target = sys.argv[1]
    root = Path(__file__).resolve().parents[2]
    with (root / "Cargo.toml").open("rb") as manifest:
        version = tomllib.load(manifest)["package"]["version"]
    name = f"mmf-v{version}-{target}"
    suffix = ".exe" if "windows" in target else ""
    output = root / "dist"
    output.mkdir(exist_ok=True)
    documents = [
        "README.md", "README_EN.md", "AGENTS.md", "LICENSE", "docs/spec.md",
        "docs/usage.md", "docs/usage_EN.md",
        ".github/workflows/release.yml", ".github/scripts/package_release.py",
    ]

    with tempfile.TemporaryDirectory(prefix="mmf-package-") as staging_dir:
        package = Path(staging_dir) / name
        package.mkdir()
        for executable in ("mmf", "mmf_shell"):
            destination = package / f"{executable}{suffix}"
            shutil.copy2(root / "target" / target / "release" / destination.name, destination)
            if not suffix:
                destination.chmod(0o755)
        for document in documents:
            destination = package / document
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(root / document, destination)

        if suffix:
            archive = output / f"{name}.zip"
            with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED) as bundle:
                for source in sorted(package.rglob("*")):
                    if source.is_file():
                        bundle.write(source, source.relative_to(package.parent))
        else:
            archive = output / f"{name}.tar.gz"
            with tarfile.open(archive, "w:gz") as bundle:
                bundle.add(package, arcname=name)

        # Extract our own trusted archive to verify permissions, contents and behavior.
        with tempfile.TemporaryDirectory(prefix="mmf-extracted-") as extracted_dir:
            if suffix:
                with zipfile.ZipFile(archive) as bundle:
                    bundle.extractall(extracted_dir)
            else:
                with tarfile.open(archive) as bundle:
                    bundle.extractall(extracted_dir, filter="data")
            verify(Path(extracted_dir) / name, suffix, version)
    print(f"Packaged and verified: {archive.name}")


if __name__ == "__main__":
    main()
