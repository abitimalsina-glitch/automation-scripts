
from pathlib import Path
import shutil
import argparse

FILE_TYPES = {
    "Documents": {
        ".pdf", ".doc", ".docx", ".txt", ".rtf",
        ".odt", ".xls", ".xlsx", ".ppt", ".pptx",
        ".csv", ".md"
    },
    "Images": {
        ".jpg", ".jpeg", ".png", ".gif", ".bmp",
        ".webp", ".svg", ".heic", ".ico"
    },
    "Videos": {
        ".mp4", ".mkv", ".mov", ".avi", ".webm"
    },
    "Audio": {
        ".mp3", ".wav", ".flac", ".aac", ".ogg", ".m4a"
    },
    "Archives": {
        ".zip", ".rar", ".7z", ".tar", ".gz", ".bz2"
    },
    "Code": {
        ".py", ".c", ".h", ".cpp", ".go", ".js", ".ts",
        ".tsx", ".jsx", ".html", ".css", ".cs", ".sh",
        ".sql", ".json", ".yaml", ".yml", ".toml"
    },
    "Installers": {
        ".dmg", ".pkg", ".exe", ".msi", ".deb", ".rpm"
    }
}


def get_category(file_path):
    extension = file_path.suffix.lower()

    for category, extensions in FILE_TYPES.items():
        if extension in extensions:
            return category

    return "Others"


def get_unique_path(destination):
    if not destination.exists():
        return destination

    parent = destination.parent
    stem = destination.stem
    suffix = destination.suffix
    counter = 1

    while True:
        new_path = parent / f"{stem}_{counter}{suffix}"

        if not new_path.exists():
            return new_path

        counter += 1


def organize_files(directory, dry_run=False):
    directory = Path(directory).expanduser().resolve()

    if not directory.is_dir():
        raise ValueError(f"Not a directory: {directory}")

    for file_path in directory.iterdir():
        # Ignore directories and symbolic links.
        if not file_path.is_file() or file_path.is_symlink():
            continue

        category = get_category(file_path)
        target_dir = directory / category
        destination = target_dir / file_path.name

        destination = get_unique_path(destination)

        if dry_run:
            print(f"[DRY RUN] {file_path.name} -> {category}/")
            continue

        target_dir.mkdir(parents=True, exist_ok=True)
        shutil.move(str(file_path), str(destination))

        print(f"Moved: {file_path.name} -> {category}/{destination.name}")


def main():
    parser = argparse.ArgumentParser(
        description="Organize files into folders by file type."
    )

    parser.add_argument(
        "directory",
        help="Directory to organize"
    )

    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Preview changes without moving files"
    )

    args = parser.parse_args()

    try:
        organize_files(args.directory, args.dry_run)
    except (ValueError, OSError) as error:
        print(f"Error: {error}")


if __name__ == "__main__":
    main()
