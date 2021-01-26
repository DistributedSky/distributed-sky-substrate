// A demonstration of interacting with custom RPCs using Polkadot js API

const {
    ApiPromise,
    WsProvider
} = require('@polkadot/api');
const dotenv = require('dotenv');
const {
    readFileSync
} = require('fs');
dotenv.config();

const ws_url = process.env.ws_url || "ws://localhost:9944";
const types = JSON.parse(readFileSync(process.env.types || "../pallets/types.json", 'utf8'));

// console.log( types );
// Construct parameters for API instance
const wsProvider = new WsProvider(ws_url);

const rpc = {}

async function main() {
    // Construct the actual api
    const api = await ApiPromise.create({
        provider: wsProvider,
        types,
        rpc,
    });

    const now = (await api.query.timestamp.now()).toNumber();
    const nextFeeMultiplier = (await api.query.transactionPayment.nextFeeMultiplier());
    const storageVersion = (await api.query.transactionPayment.storageVersion());

    console.log(`timestamp.now ${now}`);
    console.log(`transactionPayment.nextFeeMultiplier is ${nextFeeMultiplier}`);
    console.log(`transactionPayment.storageVersion is ${storageVersion}`);
}
main().catch(console.error).finally(() => process.exit());