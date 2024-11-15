.PHONY: build run image clean

build:
	cargo build --release

run:
	cargo run --release

clean:
	cargo clean

image:
	docker build -t chatig .

run-image:
	docker run -p 8081:8081 chatig

rpm:
	yum -y install rpm-build
	yum -y install rpmdevtools
	rpmdev-setuptree
	mkdir -p ../rpmbuild/{BUILD,RPMS,SOURCES,SPECS,SRPMS}
	cp -raf . ../rpmbuild/BUILD
	rpmbuild --define "_topdir `pwd`/../rpmbuild" --clean -bb chatig.spec