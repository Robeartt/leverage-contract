# Leverage Contract
```
touch .env # me might need to add secret keys here laters
docker compose up -d # or docker-compose up -d
bash run.sh
```

Inside the Docker Container:
```
cargo build --target wasm32-unknown-unknown --release
```