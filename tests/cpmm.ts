import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Cpmm } from "../target/types/cpmm";
import { initialize, setupInitializeTest } from "./utils";
import { BN } from "bn.js";
import { getAccount } from "@solana/spl-token";
import { assert } from "chai";

describe("cpmm", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const owner = anchor.Wallet.local().payer;
  console.log("owner: ", owner.publicKey.toString());

  const program = anchor.workspace.Cpmm as Program<Cpmm>;

  const confirmOptions = {
    skipPreflight: true,
  };

  it("create pool", async () => {
    const { configAddress, token0, token0Program, token1, token1Program } =
      await setupInitializeTest(
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
        { transferFeeBasisPoints: 0, MaxFee: 0 },
        confirmOptions
      );


      const initAmount0 = new BN(10000000000);
      const initAmount1 = new BN(10000000000);
      const { poolAddress, poolState } = await initialize(
        program,
        owner,
        configAddress,
        token0,
        token0Program,
        token1,
        token1Program,
        confirmOptions,
        { initAmount0, initAmount1 }
      );

      let vault0 = await getAccount(
        anchor.getProvider().connection,
        poolState.token0Vault,
        "processed",
        poolState.token0Program
      );
    
      assert.equal(vault0.amount.toString(), initAmount0.toString());

      let vault1 = await getAccount(
        anchor.getProvider().connection,
        poolState.token1Vault,
        "processed",
        poolState.token1Program
      );
      assert.equal(vault1.amount.toString(), initAmount1.toString());
  });
});
