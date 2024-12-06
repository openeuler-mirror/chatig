# kitig-k8s部署

### （零）配置Namespace

#### 1. 创建命令空间

**命名空间**：需要先创建命名空间 `ns.yaml`。

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: kitig
```

**命令：**

```bash
kubectl apply -f ns.yaml
```

#### 2. 转换镜像（可选）

可以通过以下步骤，将本地 Docker 镜像转换为 `containerd` 支持的格式：

1. **导出镜像为 tar 文件**：

   ```bash
   docker save -o kitig.tar kitig:latest
   docker save -o pgsql-kitig.tar pgsql-kitig:latest
   ```

2. **将 tar 文件传输到每个 Kubernetes 节点**（如果使用多个节点）：

3. **在每个节点导入镜像**（假设你的集群运行的是 `containerd`）：

   ```bash
   ctr -n k8s.io images import kitig.tar
   ctr -n k8s.io images import pgsql-kitig.tar
   ```

   导入成功后，镜像就可以直接在 Kubernetes 集群中使用，无需从镜像仓库拉取。

### （一）部署pgsql

#### 1.  创建 PV/PVC

创建 PV/PVC用于数据存储，确保数据在容器重启后不会丢失。

**（1）首先创建PV，以下仅供参考：**

```yaml
# pgsql-pv.yaml，用于定义数据库的物理存储
apiVersion: v1
kind: PersistentVolume
metadata:
  name: xinference-pv
spec:
  capacity:
    storage: 1Gi
  volumeMode: Filesystem
  accessModes:
  - ReadWriteOnce
  persistentVolumeReclaimPolicy: Delete
  storageClassName: local-storage
  local:
    path: /persist/data/xinference
  nodeAffinity:
    required:
      nodeSelectorTerms:
      - matchExpressions:
        - key: kubernetes.io/hostname
          operator: In
          values:
          - kitig001.novalocal
---
apiVersion: v1
kind: PersistentVolume
metadata:
  name: xinference-cache-pv
spec:
  capacity:
    storage: 30Gi
  volumeMode: Filesystem
  accessModes:
  - ReadWriteOnce
  persistentVolumeReclaimPolicy: Delete
  storageClassName: local-storage
  local:
    path: /persist/data/cache
  nodeAffinity:
    required:
      nodeSelectorTerms:
      - matchExpressions:
        - key: kubernetes.io/hostname
          operator: In
          values:
          - kitig001.novalocal

```

创建PV：

```bash
kubectl apply -f pgsql-pv.yaml
```

#### 2. 部署pgsql

以下是 `pgsql-kitig` 的 Kubernetes Deployment 资源清单，通过 NodePort 暴露服务：

```yaml
# kitig-pvc.yaml，请求 PV 的存储资源
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: pgsql-data-pvc
  namespace: kitig
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 1Gi
  storageClassName: local-storage
---
# pgsql-service.yaml, pgsql Deployment 配置
apiVersion: apps/v1
kind: Deployment
metadata:
  name: pgsql-kitig
  namespace: kitig
spec:
  replicas: 1
  selector:
    matchLabels:
      app: pgsql-kitig
  template:
    metadata:
      labels:
        app: pgsql-kitig
    spec:
      containers:
        - name: pgsql-kitig
          image: docker.io/library/pgsql-kitig:latest
          imagePullPolicy: IfNotPresent
          ports:
            - containerPort: 5432
          env:
            - name: POSTGRES_USER
              value: "kitig"
            - name: POSTGRES_PASSWORD
              value: "kitig"
            - name: POSTGRES_DB
              value: "kitig"
          volumeMounts:
            - mountPath: "/var/lib/postgresql/data"
              name: pgsql-data
      volumes:
        - name: pgsql-data
          persistentVolumeClaim:
            claimName: pgsql-data-pvc
      nodeSelector:
        kubernetes.io/hostname: cos12  # 绑定到含 GPU 的工作节点
---
# pgsql Service 配置
apiVersion: v1
kind: Service
metadata:
  name: pgsql-kitig-service
  namespace: kitig
spec:
  ports:
    - port: 5432        # 将 Service 的对外端口改为 5433
      targetPort: 5432  # 指向容器的 5432 端口
      nodePort: 30432
  selector:
    app: pgsql-kitig
  type: NodePort
```

部署 pgsql-kitig，并暴露服务：

```bash
kubectl apply -f pgsql-service.yaml
```



### （二）部署 Kitig

以下是 `kitig` 的 Kubernetes Deployment 资源清单，通过 NodePort 暴露服务：

```yaml
# Rust 应用 Deployment 配置
apiVersion: apps/v1
kind: Deployment
metadata:
  name: kitig
  namespace: kitig
spec:
  replicas: 1
  selector:
    matchLabels:
      app: kitig
  template:
    metadata:
      labels:
        app: kitig
    spec:
      initContainers:    # 作用：将容器内的配置文件，复制到主机上。
        - name: init-copy-configs
          image: docker.io/library/kitig:latest
          imagePullPolicy: IfNotPresent
          command: ["/bin/sh", "-c"]
          args:
            - |
              mkdir -p /persist/data/kitig/kitig_configs && \
              cp -r /app/src/configs/* /persist/data/kitig/kitig_configs/
          volumeMounts:
            - name: kitig-configs
              mountPath: "/persist/data/kitig/kitig_configs"
      containers:
        - name: kitig
          image: docker.io/library/kitig:latest
          imagePullPolicy: IfNotPresent
          ports:
            - containerPort: 8081
          env:
            - name: DATABASE_URL
              value: "postgres://kitig:kitig@pgsql-kitig-service.kitig.svc.cluster.local:5432/kitig"  # 使用 5433 端口
          volumeMounts:
            - mountPath: "/app/src/configs"
              name: kitig-configs
      volumes:
        - name: kitig-configs
          hostPath:
            path: "/persist/data/kitig/kitig_configs"
            type: Directory

      nodeSelector:
        kubernetes.io/hostname: cos12  # 绑定到含 GPU 的工作节点
---
# Rust 应用 Service 配置
apiVersion: v1
kind: Service
metadata:
  name: kitig-service
  namespace: kitig
spec:
  ports:
    - port: 8081
      targetPort: 8081
      nodePort: 30081
  selector:
    app: kitig
  type: NodePort  # 可配置为 LoadBalancer 以便外部访问

```

部署这些服务：

```bash
kubectl apply -f kitig.yaml
```



### （三）验证部署状态

检查应用是否成功运行。

```bash
kubectl get all -n kitig
```

你可以通过工作节点的 IP 地址和 NodePort 访问服务。例如：

- **Kitig:** `http://<Node-IP>:8081`

























