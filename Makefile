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
	
.PHONY: test
test: ## Run unit tests
	cargo test
	
.PHONY: check
check: ## Run clippy
	cargo check
	
.PHONY: cq-check
cq-check: check test ## Run code quality checks
	

.PHONY: play-solo
play-solo: ## Play a solo game locally
	battlesnake play --name solo_snake --url "http://localhost:8000" -g solo -v

.PHONY: play-solo-browser
play-solo-browser: ## Play a solo game locally, then open the replay in the browser
	battlesnake play --name solo_snake --url "http://localhost:8000" -g solo --browser

.PHONY: play-live
play-live: ## Play a versus game against the live version
	battlesnake play --name live_snake --url "http://rustapi.ndsquared.net" --name local_snake --url "http://localhost:8000" -v

.PHONY: play-live-browser
play-live-browser: ## Play a versus game against the live version, then open the replay in the browser
	battlesnake play --name live_snake --url "http://rustapi.ndsquared.net" --name local_snake --url "http://localhost:8000" --browser

