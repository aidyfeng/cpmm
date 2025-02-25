import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Cpmm } from "../target/types/cpmm";
import {
    calculateFee,
    calculatePreFeeAmount,
    deposit,
    getUserAndPoolVaultAmount,
    setupDepositTest,
  } from "./utils";
import { BN } from "bn.js";
import { assert } from "chai";

describe("deposit test", () => {
    anchor.setProvider(anchor.AnchorProvider.env());
    const owner = anchor.Wallet.local().payer;
  
    const program = anchor.workspace.Cpmm as Program<Cpmm>;
  
    const confirmOptions = {
      skipPreflight: true
    };
  
    it("deposit test",async () => {
            /// deposit without fee
        const { poolAddress, poolState } = await setupDepositTest(
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

        const {
            onwerToken0Account: ownerToken0AccountBefore,
            onwerToken1Account: ownerToken1AccountBefore,
            poolVault0TokenAccount: poolVault0TokenAccountBefore,
            poolVault1TokenAccount: poolVault1TokenAccountBefore,
        } = await getUserAndPoolVaultAmount(
            owner.publicKey,
            poolState.token0Mint,
            poolState.token0Program,
            poolState.token1Mint,
            poolState.token1Program,
            poolState.token0Vault,
            poolState.token1Vault
        );


        const liquidity = new BN(10000000000);
        await deposit(
          program,
          owner,
          0,
          poolState.token0Mint,
          poolState.token1Mint,
          liquidity,
          new BN(10000000000),
          new BN(20000000000),
          confirmOptions
        );
        const newPoolState = await program.account.poolState.fetch(poolAddress);
        assert.equal(newPoolState.lpSupply.toNumber(),liquidity.add(poolState.lpSupply).toNumber());

        const {
            onwerToken0Account: ownerToken0AccountAfter,
            onwerToken1Account: ownerToken1AccountAfter,
            poolVault0TokenAccount: poolVault0TokenAccountAfter,
            poolVault1TokenAccount: poolVault1TokenAccountAfter,
          } = await getUserAndPoolVaultAmount(
            owner.publicKey,
            poolState.token0Mint,
            poolState.token0Program,
            poolState.token1Mint,
            poolState.token1Program,
            poolState.token0Vault,
            poolState.token1Vault
          );

        const input_token0_amount =
        ownerToken0AccountBefore.amount - ownerToken0AccountAfter.amount;
        const input_token1_amount =
        ownerToken1AccountBefore.amount - ownerToken1AccountAfter.amount;
        assert.equal(
            poolVault0TokenAccountAfter.amount - poolVault0TokenAccountBefore.amount,
            input_token0_amount
        );
        assert.equal(
            poolVault1TokenAccountAfter.amount - poolVault1TokenAccountBefore.amount,
            input_token1_amount
        );
    });
});