from __future__ import annotations

from image_sdk.bailian_client import BailianClient


def test_generate_ok(requests_mock):
    api = "https://dashscope.aliyuncs.com/api/v1/services/aigc/multimodal-generation/generation"
    mocked = {
        "output": {
            "choices": [
                {
                    "finish_reason": "stop",
                    "message": {
                        "role": "assistant",
                        "content": [{"image": "https://mock/img.png"}],
                    },
                }
            ]
        }
    }
    requests_mock.post(api, json=mocked, status_code=200)
    requests_mock.get("https://mock/img.png", content=b"PNGDATA")

    c = BailianClient(api_key="dummy")
    img = c.generate("cat")
    assert img == b"PNGDATA"


