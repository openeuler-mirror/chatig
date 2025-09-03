from __future__ import annotations

import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', '..'))

from Qwen_API.image_sdk.bailian_client import BailianClient


def main() -> None:
    client = BailianClient()
    img = client.generate("一只猫坐在椅子上")
    with open("cat.png", "wb") as f:
        f.write(img)
    print("saved: cat.png")


if __name__ == "__main__":
    main()


