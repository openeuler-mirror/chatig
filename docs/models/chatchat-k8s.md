# chatchat-k8s部署

### （零）配置Namespace

为 `xinference` 和 `chatchat` 创建一个单独的命名空间，以便更好地管理。

```bash
kubectl create namespace chatchat
```

------

### （一）部署 Xinference

#### 1. 创建 PV/PVC

创建 PV/PVC用于数据存储，确保数据在容器重启后不会丢失。

**（1）首先创建PV，以下仅供参考：**

```yaml
# xinference-pv.yaml
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
kubectl apply -f xinference-pv.yaml
```

**（2）申请PVC，以下仅供参考：**

```yaml
# xinference-pvc.yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: xinference-pvc
  namespace: chatchat
spec:
  accessModes:
  - ReadWriteOnce
  resources:
    requests:
      storage: 1Gi
  storageClassName: local-storage
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: xinference-cache-pvc
  namespace: chatchat
spec:
  accessModes:
  - ReadWriteOnce
  resources:
    requests:
      storage: 30Gi
  storageClassName: local-storage

```

应用这个 PVC：

```bash
kubectl apply -f xinference-pvc.yaml
```



#### 2. 部署xinference

以下是 `xinference` 的 Kubernetes Deployment 资源清单。

```yaml
# xinference-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: xinference-deployment
  namespace: chatchat
  labels:
    app: xinference
spec:
  replicas: 1
  selector:
    matchLabels:
      app: xinference
  template:
    metadata:
      labels:
        app: xinference
    spec:
      containers:
      - name: xinference-container
        image: registry.cn-hangzhou.aliyuncs.com/xprobe_xinference/xinference:latest  # 替换<your_version>为实际版本号
        args: ["xinference-local", "-H", "0.0.0.0", "--log-level", "debug"]
        env:
        - name: XINFERENCE_MODEL_SRC
          value: "modelscope"
        volumeMounts:
        - name: xinference-storage
          mountPath: /root/.xinference
        - name: xinference-cache-storage
          mountPath: /root/.cache
        ports:
        - containerPort: 9997  # 应用在容器内监听的端口
        resources:
          limits:
            nvidia.com/gpu: 4  # 请求1个GPU，可以根据需要调整
      volumes:
      - name: xinference-storage
        persistentVolumeClaim:
          claimName: xinference-pvc
      - name: xinference-cache-storage
        persistentVolumeClaim:
          claimName: xinference-cache-pvc
      nodeSelector:
        kubernetes.io/hostname: kitig001.novalocal  # 绑定到含 GPU 的工作节点

```

部署 xinference：

```bash
kubectl apply -f xinference-deployment.yaml
```



#### 3. 创建Service

通过 NodePort 暴露服务：

```yaml
# xinference-service.yaml
apiVersion: v1
kind: Service
metadata:
  name: xinference-service
  namespace: chatchat
spec:
  type: NodePort  # 可以根据需要选择类型，如 ClusterIP, LoadBalancer
  selector:
    app: xinference
  ports:
  - port: 9997          # Service的目标端口，映射到容器的9997端口
    targetPort: 9997  
    nodePort: 30007     # 主机上的端口，可以自定义（范围30000-32767），也可以省略让K8s自动分配
```

部署这些服务：

```bash
kubectl apply -f xinference-service.yaml
```

### （二）部署 ChatChat

#### 1. 创建 PV/PVC

创建 PV/PVC用于数据存储，确保数据在容器重启后不会丢失。

**（1）首先创建PV，以下仅供参考：**

```yaml
# chatchat-pv.yaml
apiVersion: v1
kind: PersistentVolume
metadata:
  name: chatchat-pv
spec:
  capacity:
    storage: 10Gi
  volumeMode: Filesystem
  accessModes:
  - ReadWriteOnce
  persistentVolumeReclaimPolicy: Delete
  storageClassName: local-storage
  local:
    path: /persist/data/chatchat
  nodeAffinity:
    required:
      nodeSelectorTerms:
      - matchExpressions:
        - key: kubernetes.io/hostname
          operator: In
          values:
          - cos12
```

创建PV：

```bash
kubectl apply -f chatchat-pv.yaml
```

**（2）申请PVC，以下仅供参考：**

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: chatchat-pvc
  namespace: chatchat
spec:
  accessModes:
  - ReadWriteOnce
  resources:
    requests:
      storage: 10Gi
  storageClassName: local-storage
```

应用这个 PVC：

```bash
kubectl apply -f xinference-pvc.yaml
```



#### 2. 部署chatchat

以下是 `chatchat` 的 Kubernetes Deployment 资源清单。

```yaml
# chatchat-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: chatchat
  namespace: chatchat
spec:
  replicas: 1
  selector:
    matchLabels:
      app: chatchat
  template:
    metadata:
      labels:
        app: chatchat
    spec:
      containers:
      - name: chatchat
        image: ccr.ccs.tencentyun.com/langchain-chatchat/chatchat:0.3.1.3-93e2c87-20240829 
        command: ["/bin/sh", "-c"]
        args: ["python cli.py init -x http://172.25.63.12:30007/v1 -e bge-large-zh-v1.5 && python cli.py start -a"]
        volumeMounts:
        - name: chatchat-storage
          mountPath: /root/chatchat_data
        ports:
        - containerPort: 7861
        - containerPort: 8501
      volumes:
        - name: chatchat-storage
          persistentVolumeClaim:
            claimName: chatchat-pvc
      nodeSelector:
        kubernetes.io/hostname: cos12  # 将服务部署到含 GPU 的工作节点
```

部署 chatchat：

```bash
kubectl apply -f chatchat-deployment.yaml
```



#### 3. 创建 Service

由于我们在 `docker-compose` 中使用了 `network_mode: host`，在 Kubernetes 中可以通过 NodePort 或 LoadBalancer 进行暴露。

```yaml
# chatchat-service.yaml
apiVersion: v1
kind: Service
metadata:
  name: chatchat-service
  namespace: chatchat
spec:
  type: NodePort
  selector:
    app: chatchat
  ports:
  - name: http
    port: 7861
    targetPort: 7861
    nodePort: 30008  # 自定义或让K8s自动分配
  - name: grpc
    port: 8501
    targetPort: 8501
    nodePort: 30009  # 自定义或让K8s自动分配
```

部署这些服务：

```bash
kubectl apply -f chatchat-service.yaml
```



### （三）验证部署状态

检查应用是否成功运行。

```bash
kubectl get pods -n chatchat
kubectl get svc -n chatchat
```

你可以通过工作节点的 IP 地址和 NodePort 访问服务。例如：

- **Xinference:** `http://<Node-IP>:30007`
- **ChatChat:** `http://<Node-IP>:30008`，`http://<Node-IP>:30009`
