from __future__ import annotations

from typing import Optional, TypedDict


class GenerateOptions(TypedDict, total=False):
    size: str
    watermark: bool


class EditOptions(TypedDict, total=False):
    negative_prompt: str
    watermark: bool


