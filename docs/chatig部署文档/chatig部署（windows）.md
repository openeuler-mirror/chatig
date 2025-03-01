# chatig项目windows部署

##  rust环境安装

1. rust安装

   下载位置 https://www.rust-lang.org/learn/get-started

   [自动安装rustup，Rust 编译器 `rustc` 以及包管理器 `cargo`]

   ```shell
   rustc --version #出现版本信息
   cargo --version #出现版本信息
   ```

2. c++构建工具安装

   Visual Studio社区版下载位置：https://visualstudio.microsoft.com/zh-hans/downloads/

## pgsql配置

1. 安装

   使用EnterpriseDB下载PostgreSQL数据库：https://www.enterprisedb.com/downloads/postgres-postgresql-downloads。（记住密码）

   参考教程：https://www.cnblogs.com/fennudexiaohaitun/p/18517307

2. 创建数据库用户与数据库

   * 启动pqsql
   * 搜索pgAdmin 4进入图形化页面
   * 搜索SQL Shell(pgsql)进入命令行页面
     * 默认按回车
     * 直到提示输入口令则输入密码回车
     * postgres=#后输入以下指令即可

~~~shell
# 创建一个数据库用户chatig和数据库chatig
CREATE USER chatig WITH PASSWORD 'chatig';
CREATE DATABASE chatig OWNER chatig;
\q
```
~~~

3. pg_hba.conf中host默认为scram-sha-256认证无需修改，postgresql.conf中默认启用所有网络接口监听无需修改

## 编译运行

```shell
cd chatig #进入chatig目录
cargo build #编译 下载依赖包
cargo run
```

