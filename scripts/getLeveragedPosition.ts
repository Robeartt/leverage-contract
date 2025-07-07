import {
  Address,
  xdr,
  scValToNative,
  Networks,
  Keypair,
  nativeToScVal
} from "@stellar/stellar-sdk";
import {
  createToolkit,
  invokeCustomContract,
} from "soroban-toolkit";

async function getLeveragedPosition() {
  
  let adminSecret = process.env.USER_SECRET;
  if (!adminSecret) {
    throw new Error("USER_SECRET is not set");
  }
  let adminKeypair = Keypair.fromSecret(adminSecret);
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

  //
  const blendOrbitPoolId = "CAE7QVOMBLZ53CDRGK3UNRRHG5EZ5NQA7HHTFASEMYBWHG6MDFZTYHXC"
  const oUSDC = "CBZPEXQLJCGUYTAQRQ4FGCXUV5O4TZER5WSOMCGNDNIIO4EJ4FU5GQNZ"
  const USTRY = "CBLV4ATSIWU67CFSQU2NVRKINQIKUZ2ODSZBUJTJ43VJVRSBTZYOPNUR"

  try {

  //   pub struct FlashLoan {
  //     pub amount: i128,
  //     pub asset: Address,
  //     pub contract: Address,
  // }   

  const flashLoanScVal = xdr.ScVal.scvMap([
    new xdr.ScMapEntry({
      key: xdr.ScVal.scvSymbol("amount"),
      val: nativeToScVal(100, { type: "u32" }), // TODO. Put Optimal amount
    }),
    new xdr.ScMapEntry({
      key: xdr.ScVal.scvSymbol("asset"), // Asset to Borrow: oUSDC
      val: new Address(oUSDC).toScVal(),
    }),
    new xdr.ScMapEntry({
      key: xdr.ScVal.scvSymbol("contract"), // Flashloan Contract
      val: new Address(oUSDC).toScVal(),
    }),
  ])

  //  pub struct Request {
  //    pub address: Address, // asset address or liquidatee
  //    pub amount: i128,
  //    pub request_type: u32,
  // }

  //   pub fn from_u32(e: &Env, value: u32) -> Self {
  //     match value {
  //         0 => RequestType::Supply,
  //         1 => RequestType::Withdraw,
  //         2 => RequestType::SupplyCollateral,
  //         3 => RequestType::WithdrawCollateral,
  //         4 => RequestType::Borrow,
  //         5 => RequestType::Repay,
  //         6 => RequestType::FillUserLiquidationAuction,
  //         7 => RequestType::FillBadDebtAuction,
  //         8 => RequestType::FillInterestAuction,
  //         9 => RequestType::DeleteLiquidationAuction,
  //         _ => panic_with_error!(e, PoolError::BadRequest),
  //     }
  // }

  const requestsScVal = xdr.ScVal.scvVec([
    xdr.ScVal.scvMap([
      new xdr.ScMapEntry({
        key: xdr.ScVal.scvSymbol("address"), // Asset to supply as collateral. USTRY
        val: new Address(USTRY).toScVal(),
      }),
      new xdr.ScMapEntry({
        key: xdr.ScVal.scvSymbol("amount"),
        val: nativeToScVal(100, { type: "u32" }), // TODO. Put Optimal amount
      }),
      new xdr.ScMapEntry({
        key: xdr.ScVal.scvSymbol("request_type"),
        val: nativeToScVal(2, { type: "u32" }), // SupplyCollateral
      }),
    ])
  ])

  let adminScVal = new Address(adminKeypair.publicKey()).toScVal();

  //   fn flash_loan(
  //     e: Env,
  //     from: Address,
  //     flash_loan: FlashLoan,
  //     requests: Vec<Request>,
  // )
      
    const flashLoanParams : xdr.ScVal[] = [
      adminScVal,  //     from: Address,
      flashLoanScVal,                                   //     flash_loan: FlashLoan,
      requestsScVal,                                    //     requests: Vec<Request>,
    ];


    let simulate = false; 
    const result = await invokeCustomContract(
      loaded,
      blendOrbitPoolId,
      "flash_loan",
      flashLoanParams,
      simulate, // Set to false to execute the transaction
    );
    const rawScVal = result.result?.retval;
    const nativeResult = scValToNative(rawScVal);
    
    console.log("🚀 ~ nativeResult:", nativeResult)
    console.log("✅ Balance:", nativeResult.toString());

    return nativeResult;
  } catch (error) {
    console.error("❌ Error getting token balance:", error);
    return null;
  }
}

getLeveragedPosition()
