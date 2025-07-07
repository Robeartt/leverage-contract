import { 
    Keypair, Networks, rpc, Transaction, TransactionBuilder, Horizon, 
    Operation,
    Asset
} from "@stellar/stellar-sdk";
import { config } from "dotenv";
config();

import { submitTransactionWithPolling } from "./utils/submitTransactionWithPolling";



async function createTrustlines() {
    try {
      let adminSecret = process.env.USER_SECRET;
      if (!adminSecret) {
        throw new Error("USER_SECRET is not set");
      }
      let adminKeypair = Keypair.fromSecret(adminSecret);
      
      const server = new Horizon.Server('https://horizon.stellar.org');

      const USTRY = new Asset(
        'USTRY',
        'GCRYUGD5NVARGXT56XEZI5CIFCQETYHAPQQTHO2O3IQZTHDH4LATMYWC'
      );
      const oUSD = new Asset(
        'oUSD',
        'GBIWJGAOSFC4KUPHXM573TKTWHMI7VW7D4GCHYZYH243Q6HVBV7ORBIT'
      );

      const userPub = adminKeypair.publicKey();
      const userAccount = await server.loadAccount(userPub);

      const tx = new TransactionBuilder(userAccount, {
        fee: '300',
        networkPassphrase: Networks.PUBLIC,
      })
      .addOperation(Operation.changeTrust({// User sets a trustline for assetOut
          asset: USTRY,
          source: userPub,
      }))
      .addOperation(Operation.changeTrust({// User sets a trustline for assetOut
        asset: oUSD,
        source: userPub,
    }))
        .setTimeout(3000)
        .build();

      
      tx.sign(adminKeypair);

      const result = await submitTransactionWithPolling(tx);
      if (result && result.hash) {
          console.log("✅ Transaction successful:", result.hash);
      } else {
          console.log("Transaction result:", result);
      }

    } catch (error) {
        console.error('Error:', error);
    }
}

createTrustlines();
