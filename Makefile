default: frontend
	cargo build

clean:
	@cargo clean
	@yarn cache clean
	@-rm -rf browser/pkg
	@-rm ./src/cache_buster_data.json
	@-rm -rf ./static/cache/bundle
	@-rm -rf ./assets

coverage: migrate
	cargo tarpaulin -t 1200 --out Html

dev-env:
	cargo fetch
	yarn install

doc:
	cargo doc --no-deps --workspace --all-features

docker:
	docker build -t realaravinth/kaizen:master -t realaravinth/kaizen:latest .

docker-publish:
	docker push realaravinth/kaizen:master 
	docker push realaravinth/kaizen:latest

frontend:
	@yarn install
	@-rm -rf ./static/cache/bundle/
	@-mkdir ./static/cache/bundle/css/
	@yarn run dart-sass -s compressed templates/main.scss  ./static/cache/bundle/css/main.css

migrate:
	cargo run --bin tests-migrate

release: frontend
	cargo build --release

run: frontend
	cargo run

test: frontend
	echo 'static/' && tree static || true
	echo 'tree/' && tree assets || true
	cargo test --all-features --no-fail-fast

xml-test-coverage: migrate
	cargo tarpaulin -t 1200 --out Xml

help:
	@echo  '  clean                   - drop builds and environments'
	@echo  '  coverage                - build test coverage in HTML format'
	@echo  '  dev-env                 - download dependencies'
	@echo  '  docker                  - build docker image'
	@echo  '  docker-publish          - build and publish docker image'
	@echo  '  doc                     - build documentation'
	@echo  '  frontend                - build static assets in prod mode'
	@echo  '  migrate                 - run database migrations'
	@echo  '  run                     - run developer instance'
	@echo  '  test                    - run unit and integration tests'
	@echo  '  xml-coverage            - build test coverage in XML for upload to codecov'
	@echo  ''
