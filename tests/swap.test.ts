import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { Cpmm } from "../target/types/cpmm";
import { setupSwapTest, swap_base_input } from "./utils";
import { assert, config } from "chai";
import { getAccount, getAssociatedTokenAddressSync } from "@solana/spl-token";


describe("swap test", () => {
    anchor.setProvider(anchor.AnchorProvider.env());
    const owner = anchor.Wallet.local().payer;
  
    const program = anchor.workspace.Cpmm as Program<Cpmm>;
  
    const confirmOptions = {
      skipPreflight: true,
    };

    it("swap base input", async () => {
        const { configAddress, poolAddress, poolState } = await setupSwapTest(
            program,
            anchor.getProvider().connection,
            owner,
            {
              config_index: 0,
              tradeFeeRate: new BN(10),
              protocolFeeRate: new BN(1000),
              fundFeeRate: new BN(25000),
              create_fee: new BN(0),
            },
            { transferFeeBasisPoints: 0, MaxFee: 0 }
          );
        const inputToken = poolState.token0Mint;
        const inputTokenProgram = poolState.token0Program;
        const outputToken = poolState.token1Mint;
        const outputTokenProgram = poolState.token1Program;
        const inputTokenAccountAddr = getAssociatedTokenAddressSync(
            inputToken,
            owner.publicKey,
            false,
            inputTokenProgram
        );
        const outputTokenAccountAddr = getAssociatedTokenAddressSync(
            outputToken,
            owner.publicKey,
            false,
            outputTokenProgram
        );
        const inputTokenAccountBefore = await getAccount(
            anchor.getProvider().connection,
            inputTokenAccountAddr,
            "processed",
            inputTokenProgram
          );
        const outputTokenAccountBefore = await getAccount(
            anchor.getProvider().connection,
            outputTokenAccountAddr,
            "processed",
            outputTokenProgram
          );
          await sleep(1000);
          let amount_in = new BN(100000000);
          await swap_base_input(
            program,
            owner,
            0,
            inputToken,
            inputTokenProgram,
            poolState.token1Mint,
            poolState.token1Program,
            amount_in,
            new BN(0)
          );
          const inputTokenAccountAfter = await getAccount(
            anchor.getProvider().connection,
            inputTokenAccountAddr,
            "processed",
            inputTokenProgram
          );
          const outputTokenAccountAfter = await getAccount(
            anchor.getProvider().connection,
            outputTokenAccountAddr,
            "processed",
            outputTokenProgram
          );
          assert.equal(
            inputTokenAccountBefore.amount - inputTokenAccountAfter.amount,
            BigInt(amount_in.toString())
          );
          console.log("output token change",outputTokenAccountAfter.amount - outputTokenAccountBefore.amount);
    });

});

function sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }