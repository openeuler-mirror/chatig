# pgsql 常用命令


#### 1. **基本psql命令**

| **命令**                         | **描述**                   |
| -------------------------------- | -------------------------- |
| `\l`                             | 列出所有数据库             |
| `\c dbname`                      | 连接到指定的数据库         |
| `\q`                             | 退出psql命令行             |
| `\du`                            | 列出所有用户               |
| `\dt`                            | 列出当前数据库中的所有表   |
| `\d tablename`                   | 显示表的结构               |
| `\d+ tablename`                  | 查看表的详细信息           |
| `\conninfo`                      | 显示当前数据库的连接信息   |
| `\encoding`                      | 查看当前数据库的编码       |
| `\timing`                        | 开启或关闭查询执行时间显示 |
| `\pset format aligned/unaligned` | 设置输出格式               |

------

#### 2. **数据库管理命令**

| **命令**                                                  | **描述**                     |
| --------------------------------------------------------- | ---------------------------- |
| `CREATE DATABASE dbname;`                                 | 创建一个新的数据库           |
| `DROP DATABASE dbname;`                                   | 删除一个数据库               |
| `GRANT ALL PRIVILEGES ON DATABASE dbname TO username;`    | 赋予用户访问数据库的所有权限 |
| `REVOKE ALL PRIVILEGES ON DATABASE dbname FROM username;` | 撤销用户的所有权限           |
| `ALTER USER username WITH PASSWORD 'newpassword';`        | 修改用户密码                 |
| `DROP USER username;`                                     | 删除一个用户                 |

------

#### 3. **表操作命令**

| **命令**                                      | **描述**                |
| --------------------------------------------- | ----------------------- |
| `CREATE TABLE tablename (...);`               | 创建一个新表            |
| `DROP TABLE tablename;`                       | 删除表                  |
| `TRUNCATE TABLE tablename;`                   | 清空表中的所有数据      |
| `\copy tablename FROM 'file.csv' CSV HEADER;` | 从CSV文件导入数据到表   |
| `\copy tablename TO 'file.csv' CSV HEADER;`   | 导出表中的数据到CSV文件 |

------

#### 4. **数据操作命令**

| **命令**                                                  | **描述**                           |
| --------------------------------------------------------- | ---------------------------------- |
| `INSERT INTO tablename (col1, col2) VALUES (val1, val2);` | 插入数据到表中                     |
| `SELECT * FROM tablename;`                                | 查询表中的所有数据                 |
| `UPDATE tablename SET col1 = val1 WHERE condition;`       | 更新表中符合条件的数据             |
| `DELETE FROM tablename WHERE condition;`                  | 删除表中符合条件的数据             |
| `EXPLAIN SELECT * FROM tablename;`                        | 显示查询的执行计划                 |
| `\g`                                                      | 执行上一次的SQL命令                |
| `\x`                                                      | 切换扩展视图，显示更详细的查询输出 |

------

#### 5. **文件操作命令**

| **命令**             | **描述**           |
| -------------------- | ------------------ |
| `\i filepath.sql`    | 执行SQL文件        |
| `\set`               | 设置或查看psql变量 |
| `\password username` | 修改指定用户的密码 |