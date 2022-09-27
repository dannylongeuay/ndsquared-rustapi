.DEFAULT_GOAL := help

.PHONY: help
help: ## View help information
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'
	
.PHONY: asdf-bootstrap
asdf-bootstrap: ## Bootstrap tooling dependencies
	asdf plugin-add rust || asdf install rust

.PHONY: run
run: asdf-bootstrap ## Run the rust binary
	cargo run

.PHONY: build
build: asdf-bootstrap ## Build the rust binary
	cargo build

.PHONY: play-local
play-local: ## Play a versus game locally against self
	battlesnake play --name local_snake_1 --url "http://localhost:8000" --name local_snake_2 --url "http://localhost:8000" -v

.PHONY: play-local-cluster
play-local: ## Play a versus game locally with the cluster version
	battlesnake play --name cluster_snake_1 --url "http://rustapi.localhost:8000" --name cluster_snake_2 --url "http://rustapi.localhost:8000" -v

.PHONY: play-local-solo
play-local-solo: ## Play a solo game locally
	battlesnake play --name solo_snake --url "http://localhost:8000" -g solo -v

.PHONY: play-live
play-live: ## Play a versus game against the live version
	battlesnake play --name live_snake --url "http://rustapi.ndsquared.net" --name local_snake --url "http://localhost:8000" -v

.PHONY: play-live-solo
play-live-solo: ## Play a solo game with the live battlesnake version
	battlesnake play --name live_solo_snake --url "http://rustapi.ndsquared.net" -g solo -v
