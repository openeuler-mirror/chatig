#!/usr/bin/env bash
set -euo pipefail
SKIP_DEPS="${SKIP_DEPS:-0}"

### ====== 可调参数 ======
SSH_PORT="${SSH_PORT:-22}"                          # 目标机 SSH 端口（你这台内网机是 22）
CHATIG_USER="${CHATIG_USER:-chatig}"
CHATIG_BIN="${CHATIG_BIN:-/opt/chatig/bin/chatig}"  # 预期 chatig 二进制存放路径
CHATIG_DIR="${CHATIG_DIR:-/opt/chatig}"
CHATIG_ETC="${CHATIG_ETC:-/etc/chatig}"
CHATIG_PORT="${CHATIG_PORT:-8001}"
CHATIG_LISTEN="${CHATIG_LISTEN:-127.0.0.1}"
VLLM_PORT="${VLLM_PORT:-8003}"
VLLM_LISTEN="${VLLM_LISTEN:-127.0.0.1}"
CFG_FILE="${CFG_FILE:-${CHATIG_ETC}/chatig_config.yaml}"
SERVICE_FILE="${SERVICE_FILE:-/etc/systemd/system/chatig.service}"

# 0=使用你已上传的二进制；1=从源码构建
USE_BUILD_FROM_SOURCE="${USE_BUILD_FROM_SOURCE:-0}"
CHATIG_REPO_URL="${CHATIG_REPO_URL:-https://gitee.com/openeuler/chatig.git}"

### ====== 环境准备：sudo/包管理器/OS 检测 ======
if [[ "$(id -u)" -eq 0 ]]; then
  SUDO=""
else
  SUDO="sudo"
fi

PKG_MGR=""
if command -v dnf >/dev/null 2>&1; then
  PKG_MGR="dnf"
elif command -v yum >/dev/null 2>&1; then
  PKG_MGR="yum"
elif command -v apt-get >/dev/null 2>&1; then
  PKG_MGR="apt"
elif command -v zypper >/dev/null 2>&1; then
  PKG_MGR="zypper"
else
  echo "ERROR: 未检测到支持的包管理器(dnf/yum/apt/zypper)。请手工安装依赖：git curl jq 编译工具链 openssl 头文件。"
  exit 1
fi

echo "[0/8] 安装基础依赖（根据系统自动选择 ${PKG_MGR}）"
if [[ "$SKIP_DEPS" == "1" ]]; then
  echo "[0/8] 跳过依赖安装（受限网络/离线模式）"
else
  case "$PKG_MGR" in
    dnf)
      $SUDO dnf -y install git curl jq pkgconf-pkg-config openssl-devel gcc gcc-c++ make
      ;;
    yum)
      $SUDO yum -y install git curl jq pkgconfig openssl-devel gcc gcc-c++ make
      ;;
    apt)
      $SUDO apt-get update -y
      $SUDO apt-get install -y git curl jq pkg-config libssl-dev build-essential
      ;;
    zypper)
      $SUDO zypper refresh
      $SUDO zypper install -y git curl jq pkg-config libopenssl-devel gcc gcc-c++ make
      ;;
  esac
fi

### ====== 用户与目录 ======
echo "[1/8] 创建运行用户与目录"
if ! id "${CHATIG_USER}" >/dev/null 2>&1; then
  # -r 系统用户；RHEL/CentOS 下 useradd 选项兼容
  $SUDO useradd -r -m -s /sbin/nologin "${CHATIG_USER}" || $SUDO useradd -r -m -s /usr/sbin/nologin "${CHATIG_USER}" || true
fi
$SUDO mkdir -p "${CHATIG_DIR}/bin"
$SUDO mkdir -p "${CHATIG_ETC}"
$SUDO chown -R "${CHATIG_USER}:${CHATIG_USER}" "${CHATIG_DIR}" "${CHATIG_ETC}"

### ====== 安装 chatig 二进制 或 编译 ======
if [[ "${USE_BUILD_FROM_SOURCE}" == "1" ]]; then
  echo "[2/8] 从源码构建 chatig"
  if ! command -v cargo >/dev/null 2>&1; then
    echo "安装 Rust toolchain ..."
    # 对大多数企业镜像可用，如失败需手工安装
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    # shellcheck disable=SC1090
    source "$HOME/.cargo/env"
  fi
  $SUDO rm -rf /opt/chatig-src || true
  $SUDO mkdir -p /opt/chatig-src
  $SUDO chown "$(id -un)":"$(id -gn)" /opt/chatig-src
  git clone "${CHATIG_REPO_URL}" /opt/chatig-src
  cd /opt/chatig-src
  # shellcheck disable=SC1090
  source "$HOME/.cargo/env"
  cargo build --release
  $SUDO cp target/release/chatig "${CHATIG_BIN}"
  $SUDO chown "${CHATIG_USER}:${CHATIG_USER}" "${CHATIG_BIN}"
  $SUDO chmod 0755 "${CHATIG_BIN}"
else
  echo "[2/8] 使用已上传的二进制：${CHATIG_BIN}"
  if [[ ! -x "${CHATIG_BIN}" ]]; then
    echo "ERROR: 未找到可执行的 chatig 二进制：${CHATIG_BIN}"
    echo "请先上传：scp chatig <目标机>:${CHATIG_BIN} && chmod 0755"
    exit 1
  fi
fi

### ====== 写配置 ======
echo "[3/8] 写配置 ${CFG_FILE}"
$SUDO tee "${CFG_FILE}" >/dev/null <<EOF
base_config:
  listen_host: "${CHATIG_LISTEN}"
  listen_port: ${CHATIG_PORT}
  log_level: "info"
  timeout: 60.0
  max_retries: 2

modules:
  chat:
    enabled: true
    upstream:
      type: "openai_compatible"
      base_url: "http://${VLLM_LISTEN}:${VLLM_PORT}"
      api_key: ""
      default_model: "/home/aisp/project/models/Qwen3-0.6B"
  files:
    enabled: true
  embeddings:
    enabled: false
  rerank:
    enabled: false
EOF
$SUDO chown "${CHATIG_USER}:${CHATIG_USER}" "${CFG_FILE}"
$SUDO chmod 0644 "${CFG_FILE}"

### ====== systemd unit ======
echo "[4/8] 写 systemd 单元 ${SERVICE_FILE}"
$SUDO tee "${SERVICE_FILE}" >/dev/null <<EOF
[Unit]
Description=ChatIG Test Service (Local Only)
After=network.target

[Service]
User=${CHATIG_USER}
Group=${CHATIG_USER}
Environment=CHATIG_CONFIG=${CFG_FILE}
ExecStart=${CHATIG_BIN}
Restart=always
RestartSec=3
LimitNOFILE=1048576
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=full
ProtectHome=true

[Install]
WantedBy=multi-user.target
EOF

### ====== 启动 ======
echo "[5/8] 启动 chatig"
$SUDO systemctl daemon-reload
$SUDO systemctl enable --now chatig
sleep 1
systemctl status chatig --no-pager || true

### ====== 监听校验 ======
echo "[6/8] 校验监听（应为 127.0.0.1:${CHATIG_PORT}）"
if command -v ss >/dev/null 2>&1; then
  ss -ltnp | grep ":${CHATIG_PORT}" || true
else
  netstat -lntp | grep ":${CHATIG_PORT}" || true
fi

### ====== 可选：配置防火墙（RHEL/CentOS 默认 firewalld；Ubuntu 是 ufw）=====
echo "[7/8] 配置主机防火墙（仅放行 SSH ${SSH_PORT}；不开放 8001/8003）"
if command -v firewall-cmd >/dev/null 2>&1; then
  $SUDO systemctl enable --now firewalld || true
  $SUDO firewall-cmd --permanent --add-port=${SSH_PORT}/tcp
  $SUDO firewall-cmd --reload
elif command -v ufw >/dev/null 2>&1; then
  $SUDO ufw --force reset
  $SUDO ufw default deny incoming
  $SUDO ufw default allow outgoing
  $SUDO ufw allow ${SSH_PORT}/tcp
  $SUDO ufw --force enable
  $SUDO ufw status verbose
else
  echo "未检测到 firewalld/ufw，跳过。因 chatig 仅绑定 127.0.0.1，仍不会暴露公网。"
fi

### ====== 自检 ======
echo "[8/8] 健康检查（目标机本地回环）"
set +e
curl -fsS "http://127.0.0.1:${CHATIG_PORT}/health" && echo || echo "（若无 /health 路由请忽略）"
set -e
echo "✅ 完成。建议再次运行：systemctl status chatig --no-pager"
