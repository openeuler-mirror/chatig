# chatig_python_sdk/Embedding_sdk/exceptions.py

class ChatIGError(Exception):
    """Base exception for ChatIG SDK."""

class ChatIGRequestError(ChatIGError):
    """Raised when an outbound request is invalid or fails to send."""
    def __init__(self, message: str, *, endpoint: str | None = None, payload: dict | None = None):
        super().__init__(message)
        self.endpoint = endpoint
        self.payload = payload

class ChatIGResponseError(ChatIGError):
    """Raised when the remote service returns an error response."""
    def __init__(self, message: str, *, status: int | None = None, body: str | None = None):
        super().__init__(message)
        self.status = status
        self.body = body
