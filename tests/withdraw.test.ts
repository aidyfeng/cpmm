import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { Cpmm } from "../target/types/cpmm";
import {
  deposit,
  getUserAndPoolVaultAmount,
  isEqual,
  setupDepositTest,
  withdraw,
} from "./utils";
import { assert } from "chai";

describe("withdraw test",() => {
    anchor.setProvider(anchor.AnchorProvider.env());
    const owner = anchor.Wallet.local().payer;
    const program = anchor.workspace.Cpmm as Program<Cpmm>;

    const confirmOptions = {
        skipPreflight: true,
    };

    it("withdraw half of lp ", async () => {
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

        await withdraw(
            program,
            0,
            poolState.token0Mint,
            poolState.token1Mint,
            liquidity.divn(2),
            new BN(10000000),
            new BN(1000000),
            confirmOptions
        );
        const newPoolState = await program.account.poolState.fetch(poolAddress);
        assert(newPoolState.lpSupply.eq(liquidity.divn(2).add(poolState.lpSupply)));
    })
})
