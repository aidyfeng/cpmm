import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Cpmm } from "../target/types/cpmm";


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
    
    
  });
});
