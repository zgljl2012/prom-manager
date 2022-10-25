
version="v0.0.1"

build-image:
	@docker build -t prom-manager:$(version) .

push:
	@docker tag prom-manager:$(version) liaojl/prom-manager:$(version)
	@docker push liaojl/prom-manager:$(version)
