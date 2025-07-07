import {
  Address,
  xdr,
  scValToNative,
  Networks
} from "@stellar/stellar-sdk";
import {
  createToolkit,
  invokeCustomContract,
} from "soroban-toolkit";

async function getLeveragedPosition() {
  
  // admin secret should taken from .env
  let adminSecret = process.env.USER_SECRET;
  if (!adminSecret) {
    throw new Error("USER_SECRET is not set");
  }
  const toolkit = createToolkit({
        adminSecret: adminSecret,
        customNetworks: [
          {
            network: "mainnet",
            horizonRpcUrl: "https://horizon.stellar.org",
            sorobanRpcUrl: "https://mainnet.sorobanrpc.com",
            networkPassphrase: Networks.PUBLIC,
          },
        ],
        verbose: "full",
      });

  const loaded = toolkit.getNetworkToolkit("mainnet");


  // try {
  //   //export declare function invokeContract(toolkit: SorobanToolkit, contractKey: string, method: string, params: xdr.ScVal[], simulate?: boolean, source?: Keypair): Promise<any>;
    
  //   let simulate = true; // Set to true to simulate the transaction without executing it
  //   const result = await invokeCustomContract(
  //     loaded,
  //     tokenContractId,
  //     "balance",
  //     [user],
  //     simulate, // Set to false to execute the transaction
  //   );
  //   const rawScVal = result.result?.retval;
  //   const nativeResult = scValToNative(rawScVal);
    
  //   console.log("🚀 ~ nativeResult:", nativeResult)
  //   console.log("✅ Balance:", nativeResult.toString());

  //   return nativeResult;
  // } catch (error) {
  //   console.error("❌ Error getting token balance:", error);
  //   return null;
  // }
}

getLeveragedPosition()
