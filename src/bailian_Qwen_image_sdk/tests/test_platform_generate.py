from __future__ import annotations

import base64

from image_sdk.platform_client import PlatformClient


def test_platform_generate_ok(requests_mock):
    url = "http://localhost:8000/generate"
    payload = {"image_base64": base64.b64encode(b"PNGDATA").decode()}
    requests_mock.post(url, json=payload, status_code=200)

    c = PlatformClient(base_url="http://localhost:8000")
    img = c.generate("cat")
    assert img == b"PNGDATA"


