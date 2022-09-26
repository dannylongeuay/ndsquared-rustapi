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
