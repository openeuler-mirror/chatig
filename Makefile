.PHONY: build run image clean

COMMIT_ID := $(shell git rev-parse HEAD | cut -c 1-16)

build:
	cargo build --release

run:
	cargo run --release

clean:
	cargo clean

image:
	docker build -t cuig:v0.1_$(COMMIT_ID) .

run-image:
	docker run -p 8081:8081 chatig

rpm:
	yum -y install rpm-build
	yum -y install rpmdevtools
	yum -y install postgresql-devel
	rpmdev-setuptree
	mkdir -p ../rpmbuild/{BUILD,RPMS,SOURCES,SPECS,SRPMS}
	cp -raf . ../rpmbuild/BUILD
	rpmbuild --define "_topdir `pwd`/../rpmbuild" --define "version v0.1_$(COMMIT_ID)" --clean -bb chatig.spec