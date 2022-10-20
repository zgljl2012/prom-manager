
version="v0.0.1"

build-image:
	@docker build -t prom-manager:$(version) .
