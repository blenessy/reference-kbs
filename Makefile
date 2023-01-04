ARCH := amd64
VERSION := 0.3.0
REGISTRY := ghcr.io/blenessy
WORKLOAD := {"workload_id":"sevtest","tee_config":"{\"flags\":{\"bits\":63},\"minfw\":{\"major\":0,\"minor\":0}}","passphrase":"mysecretpassphrase","launch_measurement":"3c6f91219614a28d2e193e82dc2366d1a758a52c04607999b5b8ff9216304c97","affinity":["CEK EP384 E256 230f44f6705d3a0ab10edeac5aff6706713beaf3bec433d9d097b5f4c12cf5e3"]}

.PHONY: run
run:
	docker build . -t reference-kbs
	docker run --rm -p 8000:8000 --name reference-kbs -e 'WORKLOAD=$(WORKLOAD)' reference-kbs

.PHONY: push
push:
	docker build --platform $(ARCH) . -t $(REGISTRY)/reference-kbs:latest -t $(REGISTRY)/reference-kbs:$(VERSION)
	docker push $(REGISTRY)/reference-kbs:latest
	docker push $(REGISTRY)/reference-kbs:$(VERSION)
