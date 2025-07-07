# Leverage Contract
```
touch .env # me might need to add secret keys here laters
docker compose up -d # or docker-compose up -d
bash run.sh
```

Inside the Docker Container:
```
cargo build --target wasm32-unknown-unknown --release

source .env

stellar network add --rpc-url $SOROBAN_RPC --network-passphrase "Public Global Stellar Network ; September 2015" mainnet


stellar contract deploy   --wasm target/wasm32-unknown-unknown/release/leverage_contract.wasm   --source admin   --network mainnet   --alias leverage -- --collateral_asset CBLV4ATSIWU67CFSQU2NVRKINQIKUZ2ODSZBUJTJ43VJVRSBTZYOPNUR --debt_asset CBLV4ATSIWU67CFSQU2NVRKINQIKUZ2ODSZBUJTJ43VJVRSBTZYOPNUR --swap_router CBLV4ATSIWU67CFSQU2NVRKINQIKUZ2ODSZBUJTJ43VJVRSBTZYOPNUR

```

Execute the FlashLoan
```
yarn
yarn ts-node scripts/createTrustlines.ts # create trustlines to oUSD and USTRY
# send 20 USTRY to your account
yarn ts-node scripts/getLeveragedPosition.ts
```