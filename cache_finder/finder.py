import sys
from pathlib import Path


HOME = Path.home()

CACHE_ROOTS = [
    HOME / ".cache",
    HOME / "Library" / "Caches",
    HOME / ".npm",
]


def get_size(path: Path) -> int:
    total = 0

    if not path.exists():
        return 0

    try:
        for item in path.rglob("*"):
            try:
                if item.is_file():
                    total += item.stat().st_size
            except (PermissionError, FileNotFoundError):
                continue
    except (PermissionError, FileNotFoundError):
        pass

    return total


def format_size(size: int) -> str:
    units = ["B", "KB", "MB", "GB", "TB"]

    value = float(size)

    for unit in units:
        if value < 1024:
            return f"{value:.2f} {unit}"

        value /= 1024

    return f"{value:.2f} PB"


def find_caches(root: Path):
    caches = []

    if not root.exists():
        return caches

    try:
        entries = root.iterdir()
    except PermissionError:
        return caches

    for entry in entries:
        if entry.is_dir():
            size = get_size(entry)
            caches.append((entry, size))

    return sorted(caches, key=lambda x: x[1], reverse=True)


def scan_all():
    all_caches = []
    root_totals = []

    for root in CACHE_ROOTS:
        caches = find_caches(root)

        root_total = sum(size for _, size in caches)

        root_totals.append((root, root_total))

        all_caches.extend(caches)

    all_caches.sort(key=lambda x: x[1], reverse=True)

    return root_totals, all_caches


def print_normal(root_totals, all_caches):
    print("Cache Finder")
    print("─" * 70)
    print()

    print("CACHE ROOTS")
    print()

    for root, size in root_totals:
        print(f"{str(root):<50} {format_size(size):>12}")

    print()

    print("TOP CACHES")
    print()

    print(f"{'Cache':<40} {'Size':>12}")
    print("─" * 70)

    for path, size in all_caches[:10]:
        print(f"{path.name:<40} {format_size(size):>12}")

    total = sum(size for _, size in all_caches)

    print()
    print("─" * 70)
    print(f"{'TOTAL CACHE':<40} {format_size(total):>12}")


def print_verbose(root_totals, all_caches):
    print("Cache Finder")
    print("─" * 80)
    print()

    for root, root_size in root_totals:
        print(f"Cache directory: {root}")
        print()

        root_caches = [
            (path, size)
            for path, size in all_caches
            if path.parent == root
        ]

        if not root_caches:
            print("No cache directories found.")
            print()
            continue

        print(f"{'Directory':<40} {'Size':>12}")
        print("─" * 80)

        for path, size in root_caches:
            print(f"{path.name:<40} {format_size(size):>12}")
            print(f"Location: {path}")
            print()

        print("─" * 80)
        print(f"{'Directory total':<40} {format_size(root_size):>12}")
        print()

    total = sum(size for _, size in all_caches)

    print("═" * 80)
    print(f"{'TOTAL CACHE':<40} {format_size(total):>12}")
    print("═" * 80)


def main():
    verbose = "--verbose" in sys.argv or "-v" in sys.argv

    root_totals, all_caches = scan_all()

    if verbose:
        print_verbose(root_totals, all_caches)
    else:
        print_normal(root_totals, all_caches)


if __name__ == "__main__":
    main()
