# 功能介绍
程序启动生成指定数目的邀请码，管理员分配给用户邀请码，并登记信息到数据库。用户的权限通过chatig来验证。

1、程序启动创建数据库（若数据库中存在数据条目，不用重新创建数据库；没有数据则生成指定数目的以sk-开头的32位邀请码，默认设置为10）
2、接口介绍
- 为用户分配invitation code，并更新数据库；post请求invitation
- 获取数据库中所有数据；get请求invitation
- 获取指定用户信息；get请求invitation/user
- 删除指定用户，需先get请求获取用户id；delete请求invitation/{id}
- 数据库扩容缩容post请求chatig（原有数据不会被更改，给的缩容小于当前数据条数，会缩容到当前数据条数大小）

# Api 介绍
```
// 获取数据库中所有内容
curl -H "Authorization: Bearer chatig" 'http://x.x.x.x:8081/invitation'

// 获取用户信息
curl -X GET "http://x.x.x.x:8081/invitation/user" \
-H "Content-Type: application/json" \
-H "Authorization: Bearer chatig" \
-d '{"user": "张三"}'

// 根据id删除指定用户信息
curl -X DELETE -H "Authorization: Bearer chatig" "http://x.x.x.x:8081/invitation/1"

// 为用户分配邀请码，除了user其他的是选填的
curl --request POST 'http://x.x.x.x:8081/invitation' \
-H "Authorization: Bearer chatig" \
-H "Content-Type: application/json" \
-d '{
    "user": "张三",
    "origination": "联通数科",
    "telephone": "13xxxxxxxxx",
    "email": "zhangsan@chinaunicom.cn"
}'

// 更改数据库的大小，扩容和缩容
curl -X POST "http://x.x.x.x:8081/chatig" \
-H "Content-Type: application/json" \
-H "Authorization: Bearer chatig" \
-d '{"target_size": 10}'
```