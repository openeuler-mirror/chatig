# chatchat docker部署

### （一）部署xinference

#### 1. 拉取镜像

​	当前，可以通过两个渠道拉取 Xinference 的官方镜像:

1. 在 Dockerhub 的 `xprobe/xinference` 仓库里。

2. Dockerhub 中的镜像会同步上传一份到阿里云公共镜像仓库中，供访问 Dockerhub 有困难的用户拉取。拉取命令：`docker pull registry.cn-hangzhou.aliyuncs.com/xprobe_xinference/xinference:<tag>` 。

```bash
docker pull registry.cn-hangzhou.aliyuncs.com/xprobe_xinference/xinference:latest
```

**目前可用的标签包括：**

- `nightly-main`: 这个镜像会每天从 GitHub main 分支更新制作，不保证稳定可靠。
- `v<release version>`: 这个镜像会在 Xinference 每次发布的时候制作，通常可以认为是稳定可靠的。
- `latest`: 这个镜像会在 Xinference 发布时指向最新的发布版本
- 对于 CPU 版本，增加 `-cpu` 后缀，如 `nightly-main-cpu`。



#### 2. 使用镜像

​	你可以使用如下方式在容器内启动 Xinference，同时将 9997 端口映射到宿主机的 9998 端口，并且指定日志级别为 DEBUG，也可以指定需要的环境变量。

```bash
docker run -e XINFERENCE_MODEL_SRC=modelscope -p 9998:9997 --gpus all xprobe/xinference:latest xinference-local -H 0.0.0.0 --log-level debug
```

- `--gpus` 必须指定，正如前文描述，镜像必须运行在有 GPU 的机器上，否则会出现错误。
- `-H 0.0.0.0` 也是必须指定的，否则在容器外无法连接到 Xinference 服务。
- 可以指定多个 `-e` 选项赋值多个环境变量。





#### 3. 挂载模型目录

​	默认情况下，镜像中不包含任何模型文件，使用过程中会在容器内下载模型。如果需要使用已经下载好的模型，需要将宿主机的目录挂载到容器内。这种情况下，需要在运行容器时指定本地卷，并且为 Xinference 配置环境变量。

```bash
docker run -v </on/your/host>:</on/the/container> -e XINFERENCE_HOME=</on/the/container> -p 9998:9997 --gpus all xprobe/xinference:latest xinference-local -H 0.0.0.0
```

​	上述命令的原理是将主机上指定的目录挂载到容器中，并设置 `XINFERENCE_HOME` 环境变量指向容器内的该目录。这样，所有下载的模型文件将存储在您在主机上指定的目录中。您无需担心在 Docker 容器停止时丢失这些文件，下次运行容器时，您可以直接使用现有的模型，无需重复下载。

​	如果你在宿主机使用的默认路径下载的模型，由于 xinference cache 目录是用的软链的方式存储模型，需要将原文件所在的目录也挂载到容器内。例如你使用 huggingface 和 modelscope 作为模型仓库，那么需要将这两个对应的目录挂载到容器内，一般对应的 cache 目录分别在 <home_path>/.cache/huggingface 和 <home_path>/.cache/modelscope，使用的命令如下：

```bash
docker run -d \
  -v /root/.xinference:/root/.xinference \
  -v /root/.cache/huggingface:/root/.cache/huggingface \
  -v /root/.cache/modelscope:/root/.cache/modelscope \
  -p 9997:9997 \
  --gpus 2 \
  registry.cn-hangzhou.aliyuncs.com/xprobe_xinference/xinference:latest \
  xinference-local -H 0.0.0.0
```



### （二）部署chatchat

#### 1. 拉取镜像

```bash
docker pull chatimage/chatchat:0.3.1.3-93e2c87-20240829

# 国内镜像
docker pull ccr.ccs.tencentyun.com/langchain-chatchat/chatchat:0.3.1.3-93e2c87-20240829 
```

#### 2. 启动镜像

（1）下载 chatchat & xinference 启动配置文件(docker-compose.yaml)

```bash
cd ~
wget https://github.com/chatchat-space/Langchain-Chatchat/blob/master/docker/docker-compose.yaml
```

（2）启动chatchat & xinference 服务


```bash
docker-compose up -d
```

出现如下日志即为成功 ( 第一次启动需要下载 docker 镜像, 时间较长, 这里已经提前下载好了 )

```bash
WARN[0000] /root/docker-compose.yaml: `version` is obsolete 
[+] Running 2/2
 ✔ Container root-chatchat-1    Started                                                                                             0.2s 
 ✔ Container root-xinference-1  Started                   
```























