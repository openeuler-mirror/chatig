"""
ChatIG Conversation Manager - 对话管理器
提供多轮对话的会话管理功能
"""

import json
from typing import List, Optional, Dict, Any
from .models import ChatMessage, ChatCompletionRequest
from .chat_client import ChatClient


class ConversationManager:
    """
    对话管理器
    
    提供多轮对话的会话管理功能，包括：
    - 消息历史管理
    - 对话上下文维护
    - 会话状态管理
    """
    
    def __init__(self, client: ChatClient, system_prompt: Optional[str] = None):
        """
        初始化对话管理器
        
        :param client: ChatIG客户端实例
        :param system_prompt: 系统提示词
        """
        self.client = client
        self.messages: List[ChatMessage] = []
        
        # 添加系统消息
        if system_prompt:
            self.messages.append(ChatMessage.create_system_message(system_prompt))
    
    def add_message(self, role: str, content: str) -> None:
        """
        添加消息到对话历史
        
        :param role: 消息角色 (user/assistant/system)
        :param content: 消息内容
        """
        if role == "user":
            self.messages.append(ChatMessage.create_user_message(content))
        elif role == "assistant":
            self.messages.append(ChatMessage.create_assistant_message(content))
        elif role == "system":
            self.messages.append(ChatMessage.create_system_message(content))
        else:
            raise ValueError(f"Invalid role: {role}")
    
    def get_messages(self) -> List[ChatMessage]:
        """
        获取所有消息
        
        :return: 消息列表
        """
        return self.messages.copy()
    
    def clear_messages(self) -> None:
        """清空消息历史，保留系统消息"""
        # 保留系统消息
        system_messages = [msg for msg in self.messages if msg.role == "system"]
        self.messages.clear()
        self.messages.extend(system_messages)
    
    def get_conversation_summary(self) -> Dict[str, Any]:
        """
        获取对话摘要
        
        :return: 对话摘要信息
        """
        return {
            "total_messages": len(self.messages),
            "user_messages": len([msg for msg in self.messages if msg.role == "user"]),
            "assistant_messages": len([msg for msg in self.messages if msg.role == "assistant"]),
            "system_messages": len([msg for msg in self.messages if msg.role == "system"]),
            "messages": [msg.model_dump() for msg in self.messages]
        }
    
    def chat(self, message: str, model: str = "Qwen/qwen2.5-7b-instruct", **kwargs) -> str:
        """
        发送消息并获取回复
        
        :param message: 用户消息
        :param model: 模型名称
        :param kwargs: 其他参数
        :return: AI回复内容
        """
        # 添加用户消息
        self.add_message("user", message)
        
        # 发送请求
        response = self.client.create_completion(
            messages=self.messages,
            model=model,
            **kwargs
        )
        
        # 提取回复内容
        if hasattr(response, 'choices') and response.choices:
            reply_content = response.choices[0].message.content
            # 添加助手回复到历史
            self.add_message("assistant", reply_content)
            return reply_content
        else:
            raise Exception("Invalid response format")
    
    def export_conversation(self, format: str = "json") -> str:
        """
        导出对话记录
        
        :param format: 导出格式 (json/text)
        :return: 导出的对话记录
        """
        if format == "json":
            return json.dumps(self.get_conversation_summary(), ensure_ascii=False, indent=2)
        elif format == "text":
            lines = []
            for msg in self.messages:
                lines.append(f"[{msg.role.upper()}] {msg.content}")
            return "\n".join(lines)
        else:
            raise ValueError(f"Unsupported format: {format}")
    
    def import_conversation(self, data: str, format: str = "json") -> None:
        """
        导入对话记录
        
        :param data: 对话数据
        :param format: 数据格式 (json/text)
        """
        self.clear_messages()
        
        if format == "json":
            conversation_data = json.loads(data)
            for msg_data in conversation_data.get("messages", []):
                self.messages.append(ChatMessage(**msg_data))
        elif format == "text":
            lines = data.strip().split("\n")
            for line in lines:
                if line.startswith("[USER]"):
                    content = line[7:].strip()
                    self.add_message("user", content)
                elif line.startswith("[ASSISTANT]"):
                    content = line[12:].strip()
                    self.add_message("assistant", content)
                elif line.startswith("[SYSTEM]"):
                    content = line[9:].strip()
                    self.add_message("system", content)
        else:
            raise ValueError(f"Unsupported format: {format}") 