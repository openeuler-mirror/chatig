from __future__ import annotations

import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', '..'))

from Qwen_API.image_sdk.platform_client import PlatformClient


def main() -> None:
    client = PlatformClient(base_url="http://localhost:8000")
    img = client.generate("星空下的狐狸")
    with open("fox.png", "wb") as f:
        f.write(img)
    print("saved: fox.png")


if __name__ == "__main__":
    main()


