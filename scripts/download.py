"""Download GeoNames data files for global-data-tool."""
import os
import urllib.request
import zipfile

BASE_URL = "https://download.geonames.org/export/dump/"
DATA_DIR = os.path.join(os.path.dirname(__file__), "raw_data")
PROXY = "http://127.0.0.1:7897"

FILES = [
    ("cities5000.zip", "cities5000.txt"),
    ("alternateNamesV2.zip", "alternateNamesV2.txt"),
    ("countryInfo.txt", None),
]


def download_file(url: str, dest: str) -> None:
    """Download a file from URL to destination."""
    print(f"Downloading {url}...")
    proxy_handler = urllib.request.ProxyHandler({
        "http": PROXY,
        "https": PROXY,
    })
    opener = urllib.request.build_opener(proxy_handler)
    with opener.open(url) as response:
        with open(dest, "wb") as f:
            f.write(response.read())
    print(f"  Saved to {dest}")


def extract_zip(zip_path: str, extract_to: str, target_file: str) -> None:
    """Extract specific file from zip archive."""
    print(f"Extracting {target_file} from {zip_path}...")
    with zipfile.ZipFile(zip_path, "r") as zf:
        zf.extract(target_file, extract_to)
    print(f"  Extracted to {os.path.join(extract_to, target_file)}")


def main():
    os.makedirs(DATA_DIR, exist_ok=True)

    for filename, extract_target in FILES:
        file_path = os.path.join(DATA_DIR, filename)

        if not os.path.exists(file_path):
            download_file(BASE_URL + filename, file_path)

        if extract_target:
            target_path = os.path.join(DATA_DIR, extract_target)
            if not os.path.exists(target_path):
                extract_zip(file_path, DATA_DIR, extract_target)

    print("\nDownload complete! Files in:", DATA_DIR)
    print("Next step: run python build_db.py")


if __name__ == "__main__":
    main()
