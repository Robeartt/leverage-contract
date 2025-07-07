import { Horizon, TransactionBuilder, Networks } from '@stellar/stellar-sdk';


const server = new Horizon.Server('https://horizon.stellar.org');

export async function submitTransactionWithPolling(tx: any) {
  try {
    const result = await server.submitTransaction(tx);
    console.log('✅ Transaction successful:', result.hash);
    return result;
  } catch (err: any) {
    console.error('❌ Transaction submission error:', err.message);

    const txHash = err?.response?.data?.extras?.hash;
    if (txHash) {
      console.log(`⏳ Submission timeout. Polling Horizon for tx hash: ${txHash}`);

      // Poll every 3 seconds for up to 30 seconds
      const maxAttempts = 10;
      for (let attempt = 1; attempt <= maxAttempts; attempt++) {
        try {
          const txResult = await server.transactions().transaction(txHash).call();
          console.log('✅ Transaction confirmed on ledger:', txResult.ledger);
          return txResult;
        } catch (pollErr: any) {
          if (pollErr.response?.status === 404) {
            console.log(`🔁 Attempt ${attempt}: transaction not found yet...`);
            await new Promise((resolve) => setTimeout(resolve, 3000)); // wait 3s
          } else {
            console.error('❌ Unexpected polling error:', pollErr.message);
            break;
          }
        }
      }

      console.warn(`⚠️ Gave up polling after ${maxAttempts} attempts. Transaction may or may not be confirmed.`);
    } else {
      console.error('❌ No tx hash available to poll. The transaction may not have been submitted.');
    }
  }
}
