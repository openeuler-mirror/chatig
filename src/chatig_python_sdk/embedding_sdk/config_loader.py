import os
import json
from typing import Dict, Any

def load_config(config_path: str | None) -> Dict[str, Any]:
    path = config_path or os.getenv("CHATIG_CONFIG", "../config.yaml")
    if not os.path.exists(path):
        return {}
    # 尝试 YAML，再退化到 JSON
    try:
        import yaml  # pip install pyyaml
        with open(path, "r", encoding="utf-8") as f:
            return yaml.safe_load(f) or {}
    except Exception:
        try:
            with open(path, "r", encoding="utf-8") as f:
                return json.load(f)
        except Exception:
            return {}
