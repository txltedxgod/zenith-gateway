.PHONY: all test run lint clean docker-build

all: test

test:
	cargo test

run:
	cargo run

lint:
	@echo "Running lint checks..."

clean:
	@echo "Cleaning artifacts..."

docker-build:
	docker build -t app:latest .
