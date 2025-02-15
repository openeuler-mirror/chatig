# docker部署

###  （一）生成 Docker 镜像

1. **生成镜像**

​	首先，确保你在项目根目录下，并且 Dockerfile 文件位于该目录。然后，可以使用以下命令来生成 Docker 镜像：

```bash
docker build -t chatig:v0208 .
```

- `-t your_image_name`：给镜像命名为 `your_image_name`，你可以根据需求修改为任何名字。
- `.`：指示 Docker 使用当前目录中的 Dockerfile 来构建镜像。

这条命令会执行 Dockerfile 中的每一步，安装依赖、构建项目，并生成最终的镜像。

2. **导出 Docker 镜像**

你可以使用 `docker save` 命令将镜像导出为一个 tar 文件。这个命令可以将镜像保存为一个可在其他机器上导入的文件。

```bash
docker save -o my_image.tar your_image_name
```

- `-o my_image.tar`：指定导出文件的路径和文件名（例如 `my_image.tar`）。
- `your_image_name`：你想要导出的镜像的名称。

执行成功后，会在当前目录下生成一个名为 `my_image.tar` 的文件。

3.  **将导出的镜像传输到其他主机**

   将 `my_image.tar` 文件传输到目标主机上。你可以使用任何传输工具，例如 `scp`、`rsync` 或者通过 USB 等方式传输。

   例如，使用 `scp` 将文件传输到远程主机：

   ```bash
   scp my_image.tar user@remote_host:/path/to/destination
   ```

4. **在目标主机上导入 Docker 镜像**

   在目标主机上，你可以使用 `docker load` 命令来导入刚才导出的 tar 文件。

   ```bash
   docker load -i my_image.tar
   ```

   - `-i my_image.tar`：指定要导入的 tar 文件。

   这条命令会加载并恢复镜像到目标主机。



### （二）部署chatig

1.  **运行容器**

现在，你可以使用以下命令运行容器：

```bash
docker run -d -p 8081:8081 \
	-v /home/yangwb/docker_chatig/configs.yaml:/app/src/configs/configs.yaml \
	--name chatig_serve \
	chatig:v0211_http
```

- `-d`：让容器在后台运行。
- `-p <宿主机端口>:<容器端口>`： 将主机的 8081 端口映射到容器的 8081 端口，这样可以通过主机的 `8081` 端口访问应用。
- `--name your_container_name`：给容器命名，你可以自定义容器的名字。
- `your_image_name`：你之前生成的镜像的名字。

2. **查看容器运行状态**

可以使用以下命令查看容器的状态：

```bash
docker ps
```

如果容器已经启动并运行，你应该能看到它的相关信息。

3. **停止和删除容器（可选）**

如果你想停止并删除容器，可以使用以下命令：

```bash
docker stop your_container_name
docker rm your_container_name
```

4.. **查看容器日志**

如果你需要查看容器的日志输出（例如查看应用是否正常启动），可以使用以下命令：

```bash
docker logs your_container_name
```

通过这些步骤，你应该能够成功地使用 Docker 来构建和部署你的 Rust 应用。如果在执行过程中遇到任何问题，随时告诉我，我可以提供帮助！

