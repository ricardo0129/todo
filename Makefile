init:
	docker build -t my-rust-app .

start:
	docker run -p 3000:3000 -it --rm --name my-running-app my-rust-app

stop:
	docker stop my-running-app

test:
	cargo test
