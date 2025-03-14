import { BN, Program } from "@coral-xyz/anchor";
import {
  ConfirmOptions,
  Connection,
  Keypair,
  PublicKey,
  Signer,
} from "@solana/web3.js";
import { Cpmm } from "../../target/types/cpmm";
import {
  accountExist,
  createTokenMintAndAssociatedTokenAccount,
  getAmmConfigAddress,
  getPoolAddress,
  sendTransaction,
} from "./index";

export async function setupInitializeTest(
  program: Program<Cpmm>,
  connection: Connection,
  owner: Signer,
  config: {
    config_index: number;
    tradeFeeRate: BN;
    protocolFeeRate: BN;
    fundFeeRate: BN;
    create_fee: BN;
  },
  transferFeeConfig: { transferFeeBasisPoints: number; MaxFee: number } = {
    transferFeeBasisPoints: 0,
    MaxFee: 0,
  },
  confirmOptions?: ConfirmOptions
) {
  const [{ token0, token0Program }, { token1, token1Program }] =
    await createTokenMintAndAssociatedTokenAccount(
      connection,
      owner,
      new Keypair(),
      transferFeeConfig
    );
  const configAddress = await createAmmConfig(
    program,
    connection,
    owner,
    config.config_index,
    config.tradeFeeRate,
    confirmOptions
  );
  return {
    configAddress,
    token0,
    token0Program,
    token1,
    token1Program,
  };
}

export async function createAmmConfig(
  program: Program<Cpmm>,
  connection: Connection,
  owner: Signer,
  config_index: number,
  tradeFeeRate: BN,
  confirmOptions?: ConfirmOptions
): Promise<PublicKey> {
  const [address, _] = await getAmmConfigAddress(
    config_index,
    program.programId
  );
  if (await accountExist(connection, address)) {
    return address;
  }

  const ix = await program.methods
    .createAmmConfig(config_index, tradeFeeRate)
    .accounts({
      owner: owner.publicKey,
      // ammConfig: address,
      // systemProgram: SystemProgram.programId,
    })
    .instruction();

  const tx = await sendTransaction(connection, [ix], [owner], confirmOptions);
  console.log("init amm config tx: ", tx);
  return address;
}

export async function initialize(
  program: Program<Cpmm>,
  creator: Signer,
  config_index: number,
  token0: PublicKey,
  token0Program: PublicKey,
  token1: PublicKey,
  token1Program: PublicKey,
  confirmOptions?: ConfirmOptions,
  initAmount: { initAmount0: BN; initAmount1: BN } = {
    initAmount0: new BN(10000000000),
    initAmount1: new BN(20000000000),
  }
) {
  const [ammConfigAddress] = await getAmmConfigAddress(
    config_index,
    program.programId
  );
  console.log("getAmmConfigAddress:", ammConfigAddress);

  const [poolAddress] = await getPoolAddress(
    ammConfigAddress,
    token0,
    token1,
    program.programId
  );
  console.log("getPoolAddress:", poolAddress);

  /* console.log(
    "initialize1:",
    initAmount.initAmount0,
    initAmount.initAmount1,
    config_index
  );
  console.log(
    "initialize2:",
    creator.publicKey,
    token0,
    token1,
    token0Program,
    token1Program
  ); */
      const transactionSignature = await program.methods
      .initialize(
        config_index,
        initAmount.initAmount0,
        initAmount.initAmount1,
        new BN(0)
      )
      .accounts({
        creator: creator.publicKey,
        token0Mint: token0,
        token1Mint: token1,
        token0Program: token0Program,
        token1Program: token1Program,
      })
      .rpc(confirmOptions)
      .catch(err => console.error("Transaction failed!", err));

    console.log("initialize transactionSignature:", transactionSignature);


  const poolState = await program.account.poolState.fetch(poolAddress);
  return { poolAddress, poolState };
}

export async function setupDepositTest(
  program: Program<Cpmm>,
  connection: Connection,
  owner: Signer,
  config: {
    config_index: number;
    tradeFeeRate: BN;
    protocolFeeRate: BN;
    fundFeeRate: BN;
    create_fee: BN;
  },
  transferFeeConfig: { transferFeeBasisPoints: number; MaxFee: number } = {
    transferFeeBasisPoints: 0,
    MaxFee: 0,
  },
  confirmOptions?: ConfirmOptions,
  initAmount: { initAmount0: BN; initAmount1: BN } = {
    initAmount0: new BN(10000000000),
    initAmount1: new BN(20000000000),
  },
  tokenProgramRequired?: {
    token0Program: PublicKey;
    token1Program: PublicKey;
  }
) {
  const configAddress = await createAmmConfig(
    program,
    connection,
    owner,
    config.config_index,
    config.tradeFeeRate,
    confirmOptions
  );

  while (1) {
    const [{ token0, token0Program }, { token1, token1Program }] =
      await createTokenMintAndAssociatedTokenAccount(
        connection,
        owner,
        new Keypair(),
        transferFeeConfig
      );

    if (tokenProgramRequired != undefined) {
      if (
        token0Program.equals(tokenProgramRequired.token0Program) &&
        token1Program.equals(tokenProgramRequired.token1Program)
      ) {
        return await initialize(
          program,
          owner,
          config.config_index,
          token0,
          token0Program,
          token1,
          token1Program,
          confirmOptions,
          initAmount
        );
      }
    } else {
      return await initialize(
        program,
        owner,
        config.config_index,
        token0,
        token0Program,
        token1,
        token1Program,
        confirmOptions,
        initAmount
      );
    }
  }
}

export async function deposit(
  program: Program<Cpmm>,
  owner: Signer,
  config_index: number,
  token0Mint: PublicKey,
  token0Program: PublicKey,
  token1Mint: PublicKey,
  token1Program: PublicKey,
  lp_token_amount: BN,
  maximum_token_0_amount: BN,
  maximum_token_1_amount: BN,
  confirmOptions?: ConfirmOptions
) {
  const tx = await program.methods
    .deposit(
      config_index,
      lp_token_amount,
      maximum_token_0_amount,
      maximum_token_1_amount
    )
    .accounts({
      owner: owner.publicKey,
      token0Mint: token0Mint,
      token1Mint: token1Mint,
      token0Program: token0Program,
      token1Program: token1Program,
    })
    .rpc(confirmOptions);

  console.log("deposit tx:", tx);
  return tx;
}

export async function withdraw(
  program: Program<Cpmm>,
  config_index: number,
  token0: PublicKey,
  token0Program: PublicKey,
  token1: PublicKey,
  token1Program: PublicKey,
  lp_token_amount: BN,
  minimum_token_0_amount: BN,
  minimum_token_1_amount: BN,
  confirmOptions?: ConfirmOptions
) {
  const tx = await program.methods
    .withdraw(
      config_index,
      lp_token_amount,
      minimum_token_0_amount,
      minimum_token_1_amount
    )
    .accounts({
      // owner: owner.publicKey,
      vault0Mint: token0,
      token0Program: token0Program,
      vault1Mint: token1,
      token1Program: token1Program,
    })
    .rpc(confirmOptions)
    .catch();

  console.log("withdraw tx", tx);

  return tx;
}

export async function setupSwapTest(
  program: Program<Cpmm>,
  connection: Connection,
  owner: Signer,
  config: {
    config_index: number;
    tradeFeeRate: BN;
    protocolFeeRate: BN;
    fundFeeRate: BN;
    create_fee: BN;
  },
  transferFeeConfig: { transferFeeBasisPoints: number; MaxFee: number } = {
    transferFeeBasisPoints: 0,
    MaxFee: 0,
  },
  confirmOptions?: ConfirmOptions
) {
  const configAddress = await createAmmConfig(
    program,
    connection,
    owner,
    config.config_index,
    config.tradeFeeRate,
    confirmOptions
  );

  const [{ token0, token0Program }, { token1, token1Program }] =
    await createTokenMintAndAssociatedTokenAccount(
      connection,
      owner,
      new Keypair(),
      transferFeeConfig
    );

  const { poolAddress, poolState } = await initialize(
    program,
    owner,
    config.config_index,
    token0,
    token0Program,
    token1,
    token1Program,
    confirmOptions
  );

  await deposit(
    program,
    owner,
    config.config_index,
    poolState.token0Mint,
    poolState.token0Program,
    poolState.token1Mint,
    poolState.token1Program,
    new BN(10000000000),
    new BN(100000000000),
    new BN(100000000000),
    confirmOptions
  );
  return { configAddress, poolAddress, poolState };
}

export async function swap_base_input(
  program: Program<Cpmm>,
  owner: Signer,
  config_index: number,
  inputToken: PublicKey,
  inputTokenProgram: PublicKey,
  outputToken: PublicKey,
  outputTokenProgram: PublicKey,
  amount_in: BN,
  minimum_amount_out: BN,
  confirmOptions?: ConfirmOptions
) {
  const tx = await program.methods
    .swapBaseInput(config_index, amount_in, minimum_amount_out)
    .accounts({
      payer: owner.publicKey,
      // authority: auth,
      // ammConfig: configAddress,
      // poolState: poolAddress,
      // inputTokenAccount,
      // outputTokenAccount,
      // inputVault,
      // outputVault,
      inputTokenProgram: inputTokenProgram,
      outputTokenProgram: outputTokenProgram,
      inputTokenMint: inputToken,
      outputTokenMint: outputToken,
      // observationState: observationAddress,
    })
    .rpc(confirmOptions);

  return tx;
}

export async function swap_base_output(
  program: Program<Cpmm>,
  owner: Signer,
  config_index: number,
  inputToken: PublicKey,
  inputTokenProgram: PublicKey,
  outputToken: PublicKey,
  outputTokenProgram: PublicKey,
  amount_out_less_fee: BN,
  max_amount_in: BN,
  confirmOptions?: ConfirmOptions
) {
  const tx = await program.methods
    .swapBaseOutput(config_index, amount_out_less_fee, max_amount_in)
    .accounts({
      payer: owner.publicKey,
      inputTokenProgram: inputTokenProgram,
      outputTokenProgram: outputTokenProgram,
      inputTokenMint: inputToken,
      outputTokenMint: outputToken,
    })
    .rpc(confirmOptions);

  return tx;
}
