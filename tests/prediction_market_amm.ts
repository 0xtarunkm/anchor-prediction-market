import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PredictionMarketAmm } from "../target/types/prediction_market_amm";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createMint,
  getAssociatedTokenAddressSync,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { PublicKey, SendTransactionError } from "@solana/web3.js";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";

describe("prediction_market", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace
    .PredictionMarketAmm as Program<PredictionMarketAmm>;

  const providerWallet = provider.wallet as NodeWallet;

  let mintYes: PublicKey;
  let mintNo: PublicKey;
  let mintUSDC: PublicKey;
  let mintLP: PublicKey;
  let market: PublicKey;
  let vaultYes: PublicKey;
  let vaultNo: PublicKey;
  let vaultUSDC: PublicKey;
  let userAtaUSDC: PublicKey;
  let userAtaYes: PublicKey;
  let userAtaNo: PublicKey;
  let userAtaLP: PublicKey;

  const seed = new anchor.BN(Math.floor(Math.random() * 1000000));
  const marketName = "VK_100_IND_BAN_2024";
  const fee = 100;
  const endTime = new anchor.BN(Math.floor(Date.now() / 1000) + 86400);

  it("Airdrop SOL to user", async () => {
    const tx = await provider.connection.requestAirdrop(
      providerWallet.publicKey,
      1000000000
    );
    await provider.connection.confirmTransaction(tx);
    console.log(
      "User balance:",
      await provider.connection.getBalance(providerWallet.publicKey)
    );
  });

  it("Initialize the market and mint tokens", async () => {
    const [marketPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("market"), seed.toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    market = marketPda;

    const [mintLp] = PublicKey.findProgramAddressSync(
      [Buffer.from("lp"), market.toBytes()],
      program.programId
    );

    mintLP = mintLp;

    userAtaLP = getAssociatedTokenAddressSync(
      mintLP,
      providerWallet.publicKey,
      false,
      TOKEN_PROGRAM_ID
    );

    mintYes = await createMint(
      provider.connection,
      providerWallet.payer,
      market,
      null,
      6
    );

    mintNo = await createMint(
      provider.connection,
      providerWallet.payer,
      market,
      null,
      6
    );

    mintUSDC = await createMint(
      provider.connection,
      providerWallet.payer,
      providerWallet.publicKey,
      null,
      6
    );

    vaultYes = getAssociatedTokenAddressSync(
      mintYes,
      market,
      true,
      TOKEN_PROGRAM_ID
    );

    vaultNo = getAssociatedTokenAddressSync(
      mintNo,
      market,
      true,
      TOKEN_PROGRAM_ID
    );

    vaultUSDC = getAssociatedTokenAddressSync(
      mintUSDC,
      market,
      true,
      TOKEN_PROGRAM_ID
    );

    try {
      const tx = await program.methods
        .initialize(seed, marketName, fee, endTime)
        .accountsPartial({
          mintYes,
          mintNo,
          mintUsdc: mintUSDC,
          signer: providerWallet.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          market,
          vaultNo,
          vaultUsdc: vaultUSDC,
          vaultYes,
        })
        .rpc();

      console.log("Market initialized, transaction signature:", tx);
    } catch (error) {
      if (error instanceof SendTransactionError) {
        console.error("Transaction failed:", error.message);
        console.error("Logs:", error.logs);
      } else {
        console.error("An unexpected error occurred:", error);
      }
      throw error;
    }
  });

  it("Initialize market with 1M shares on each side", async () => {
    userAtaUSDC = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        providerWallet.payer,
        mintUSDC,
        providerWallet.publicKey
      )
    ).address;

    await mintTo(
      provider.connection,
      providerWallet.payer,
      mintUSDC,
      userAtaUSDC,
      providerWallet.publicKey,
      2_000_000_000
    );

    try {
      const tx = await program.methods
        .addLiquidity(
          new anchor.BN(1_000_000_000),
          new anchor.BN(1_000_000_000),
          new anchor.BN(Math.floor(Date.now() / 1000) + 3600)
        )
        .accountsStrict({
          market,
          mintNo,
          mintYes,
          userAtaLp: userAtaLP,
          vaultNo,
          vaultUsdc: vaultUSDC,
          vaultYes,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          mintLp: mintLP,
          mintUsdc: mintUSDC,
          user: providerWallet.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([providerWallet.payer])
        .rpc({ skipPreflight: true });

      const vaultUSDCBalance = await provider.connection.getTokenAccountBalance(
        vaultUSDC
      );
      const vaultYesBalance = await provider.connection.getTokenAccountBalance(
        vaultYes
      );
      const vaultNoBalance = await provider.connection.getTokenAccountBalance(
        vaultNo
      );

      console.log("Initial market state:");
      console.log(`USDC in vault: ${vaultUSDCBalance.value.uiAmount}`);
      console.log(`YES tokens in vault: ${vaultYesBalance.value.uiAmount}`);
      console.log(`NO tokens in vault: ${vaultNoBalance.value.uiAmount}`);
      console.log("Liquidity added, transaction signature:", tx);
    } catch (error) {
      if (error instanceof SendTransactionError) {
        console.error("Transaction failed:", error.message);
        console.error("Logs:", error.logs);
      } else {
        console.error("An unexpected error occurred:", error);
      }
      throw error;
    }
  });

  it("Test LMSR pricing with multiple swaps", async () => {
    userAtaYes = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        providerWallet.payer,
        mintYes,
        providerWallet.publicKey
      )
    ).address;

    userAtaNo = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        providerWallet.payer,
        mintNo,
        providerWallet.publicKey
      )
    ).address;

    await mintTo(
      provider.connection,
      providerWallet.payer,
      mintUSDC,
      userAtaUSDC,
      providerWallet.publicKey,
      500_000_000
    );

    const swapTests = [
      { amount: 10_000_000, isYes: true, description: "10 USDC YES buy" },
      { amount: 5_000_000, isYes: false, description: "5 USDC NO buy" },
      { amount: 2_500_000, isYes: true, description: "2.5 USDC YES buy" },
    ];

    for (const test of swapTests) {
      console.log(`\nExecuting ${test.description}`);
      console.log("Before swap balances:");
      await logBalances();

      try {
        const tx = await program.methods
          .swap(
            true,
            new anchor.BN(test.amount),
            test.isYes,
            new anchor.BN(1),
            new anchor.BN(Math.floor(Date.now() / 1000) + 60)
          )
          .accountsStrict({
            userAtaNo,
            market,
            mintLp: mintLP,
            mintNo,
            mintUsdc: mintUSDC,
            mintYes,
            user: providerWallet.publicKey,
            userAtaUsdc: userAtaUSDC,
            userAtaYes,
            vaultNo,
            vaultUsdc: vaultUSDC,
            vaultYes,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([providerWallet.payer])
          .rpc();

        console.log("After swap balances:");
        await logBalances();
        console.log(`Swap completed, tx: ${tx}`);
      } catch (error) {
        console.error(`Error in ${test.description}:`, error);
        throw error;
      }
    }
  });

  async function logBalances() {
    const vaultUSDCBalance = await provider.connection.getTokenAccountBalance(
      vaultUSDC
    );
    const vaultYesBalance = await provider.connection.getTokenAccountBalance(
      vaultYes
    );
    const vaultNoBalance = await provider.connection.getTokenAccountBalance(
      vaultNo
    );
    const userUSDCBalance = await provider.connection.getTokenAccountBalance(
      userAtaUSDC
    );
    const userYesBalance = await provider.connection.getTokenAccountBalance(
      userAtaYes
    );
    const userNoBalance = await provider.connection.getTokenAccountBalance(
      userAtaNo
    );

    console.log("Market Balances:");
    console.log(`USDC in vault: ${vaultUSDCBalance.value.uiAmount}`);
    console.log(`YES tokens in vault: ${vaultYesBalance.value.uiAmount}`);
    console.log(`NO tokens in vault: ${vaultNoBalance.value.uiAmount}`);
    console.log("User Balances:");
    console.log(`USDC: ${userUSDCBalance.value.uiAmount}`);
    console.log(`YES tokens: ${userYesBalance.value.uiAmount}`);
    console.log(`NO tokens: ${userNoBalance.value.uiAmount}`);
  }
});
