from flask import Flask, request, Response
import requests, os, json

UPSTREAM = "https://aigw-nmhhht.cucloud.cn/v1/embeddings"
API_KEY = "sk-gw2m0VxGMSjXqG37TWQXlCGs0Wz6wDPe"

app = Flask(__name__)

@app.post("/v1/embeddings")
def embeddings():
    data = request.get_json(force=True)
    model = data.get("model")
    enc = data.get("encoding_format")

    # 统一成上游期望：inputs = List[str]
    if "inputs" in data:
        inputs = data["inputs"]
        if isinstance(inputs, str):
            inputs = [inputs]
    else:
        inp = data.get("input", [])
        inputs = [inp] if isinstance(inp, str) else inp

    payload = {"model": model, "inputs": inputs}
    if enc is not None:
        payload["encoding_format"] = enc

    r = requests.post(
        UPSTREAM, json=payload,
        headers={
            "Content-Type": "application/json",
            "Authorization": f"Bearer {API_KEY}",
        }, timeout=60
    )
    return Response(r.content, status=r.status_code, content_type=r.headers.get("Content-Type","application/json"))

if __name__ == "__main__":
    app.run("127.0.0.1", 8899)
