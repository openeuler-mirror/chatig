from .client import EmbeddingsClient

cli = EmbeddingsClient(
    base_url="http://127.0.0.1:8000",
    api_key="",  # 如不需要可留空
)

res = cli.create(
    # 模型不支持
    model="/home/aisp/project/models/Qwen3-32B",
    input=["今天天气不错", "适合散步"],  # 也可传单个 str
)
print("model:", res.model)
print("dim:", len(res.data[0].embedding))
print("usage:", res.usage.total_tokens)
