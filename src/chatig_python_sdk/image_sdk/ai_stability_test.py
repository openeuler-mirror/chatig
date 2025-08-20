from image_sdk import create_stability_client, ImageModelType, ImageSize

# 使用你的密钥
client = create_stability_client("sk-x7PcgaZEUNuf5GExcrenQybRPLScVyg16yEmlS4ICnfaG4Ae")

# 测试连接
print("连接测试:", client.test_connection())

# 生成图片
# 修改提示词为更简洁格式（避免特殊符号）
response = client.generate_image(
    "A red apple on a table",  # 简化提示词
    model=ImageModelType.STABILITY_SDXL,
    size=ImageSize.LARGE,  # 测试1024x1024尺寸
    save_local=True
)

print("生成成功:", response.model)
client.close()