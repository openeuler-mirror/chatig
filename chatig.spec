Name:           chatig
Version:        1.0
Release:        1%{?dist}
Summary:        Universal API

License:        MIT

%description
Universal API for models.

%prep
if ! command -v rustc &> /dev/null; then
    echo "Rust is not installed. Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
fi

%build
cargo build --release

%install
mkdir -p %{buildroot}/etc/chatig
mkdir -p %{buildroot}/usr/local/bin/
install -m 644 src/configs/configs.yaml %{buildroot}/etc/chatig/
install -m 644 src/configs/servers_configs.yaml %{buildroot}/etc/chatig/
install -m 755 target/release/chatig %{buildroot}/usr/local/bin/

mkdir -p %{buildroot}/usr/lib/systemd/system/
cat > %{buildroot}/usr/lib/systemd/system/chatig.service << EOF
[Unit]
Description=chatig Service
After=network.target

[Service]
ExecStart=/usr/local/bin/chatig
Restart=always

[Install]
WantedBy=multi-user.target
EOF

%files
/usr/local/bin/chatig
/usr/lib/systemd/system/chatig.service
/etc/chatig/configs.yaml
/etc/chatig/servers_configs.yaml

%post
sudo systemctl daemon-reload  
sudo systemctl start chatig.service
sudo systemctl enable chatig.service

%changelog
* Thu Nov 07 2024 - Initial release
- Initial package.