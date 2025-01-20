#! /bin/bash

# 获取git根目录
ROOT_DIR=$(git rev-parse --show-toplevel)

# 查看仓库中的rust代码行数（排除debug目录）
find $ROOT_DIR -type f -name "*.rs" |grep -v debug| xargs wc -l



