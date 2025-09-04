from __future__ import annotations

import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', '..'))

from Qwen_API.image_sdk.bailian_client import BailianClient


def main() -> None:
    client = BailianClient()
    with open("input.jpg", "rb") as f:
        src = f.read()
    img = client.edit(src, "将图中的猫改成黑色猫")
    with open("edited.png", "wb") as f:
        f.write(img)
    print("saved: edited.png")


if __name__ == "__main__":
    main()


