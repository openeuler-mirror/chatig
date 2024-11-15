# Pgsql环境配置

### （一）pgsql配置

#### 1. 手动安装配置（可选）
**（1）安装postgresql数据库**

```bash
# 安装依赖包
yum install -y postgresql-server postgresql

# 初始化数据库
/usr/bin/postgresql-setup --initdb

# 服务自启动
systemctl start postgresql
systemctl enable postgresql
```

**（2）创建数据库用户与数据库**

```bash
# 切换到PostgreSQL用户并进入psql命令行：
sudo -i -u postgres
psql

# 创建一个数据库用户和数据库
CREATE USER chatig WITH PASSWORD 'chatig';
CREATE DATABASE chatig OWNER chatig;
\q
```

**（3）配置PostgreSQL允许外部连接**

修改pg_hba.conf，将host设置为md5认证：
```bash
vim /var/lib/pgsql/data/pg_hba.conf

# IPv4 local connections:
host    all             all             127.0.0.1/32            md5
# IPv6 local connections:
host    all             all             ::1/128                 md5
```

修改postgresql.conf，确保你已经启用了所有网络接口的监听（允许外部连接）。检查或添加：
```bash
vim /var/lib/pgsql/data/postgresql.conf

listen_addresses = '*'
```

#### 2. docker部署（推荐）

```bash
# 构建 PostgreSQL 镜像
docker build -t pgsql-chatig -f Dockerfile.pgsql .

# 运行容器
docker run -d -p 5432:5432 --name pgsql-chatig pgsql-chatig
```

#### 3. 登录验证

​	尝试从远程或本地连接到PostgreSQL实例。假设PostgreSQL服务器的IP地址是`192.168.1.100`，使用如下命令测试连接：

```bash
# 在本地连接
psql -h localhost -p 5432 -U chatig -d chatig
```

输入密码`chatig`，成功连接会显示数据库提示符。