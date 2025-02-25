# Pgsql环境配置

### （一）pgsql配置

#### 1. 手动安装配置（可选）
**（1）安装postgresql数据库**

```bash
# 安装依赖包
sudo apt update
sudo apt install postgresql postgresql-contrib

# 服务自启动
systemctl status postgresql
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

exit
```

**（3）配置PostgreSQL允许外部连接**

修改pg_hba.conf，将host设置为md5认证：

```bash
# ubuntu series
vim /etc/postgresql/14/main/pg_hba.conf

# IPv4 local connections:
host    all             all             127.0.0.1/32            md5
# IPv6 local connections:
host    all             all             ::1/128                 md5
```

**（4）添加特定ip访问权限（可选）**

1. 使用宿主机的内网 IP 地址

确保 PostgreSQL 配置 (`pg_hba.conf`) 允许来自 Docker 容器的连接（即宿主机的内网 IP 地址）。

- 在 `pg_hba.conf` 中添加类似如下规则：

```bash
host    chatig    chatig    192.168.0.168/24    md5
host    chatig    chatig    172.17.0.4/24       md5
```

  这条规则允许 IP 地址 `192.168.0.168` 连接 `chatig` 数据库，并使用 MD5 验证。

- 在 Docker 容器内，使用 `psql` 或其他 PostgreSQL 客户端连接到宿主机的数据库：

  ```bash
  psql -h 192.168.0.168 -U chatig -d chatig
  ```

修改完 `pg_hba.conf` 后，需要重新加载 PostgreSQL 配置使其生效：

```bash
sudo systemctl reload postgresql
```

**（5）修改postgresql.conf配置**

修改postgresql.conf，确保你已经启用了所有网络接口的监听（允许外部连接）。检查或添加：

```bash
vim /etc/postgresql/14/main/postgresql.conf

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