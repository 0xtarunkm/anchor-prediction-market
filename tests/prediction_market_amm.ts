import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PredictionMarketAmm } from "../target/types/prediction_market_amm";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  PublicKey,
  SendTransactionError,
} from "@solana/web3.js";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import { MPL_TOKEN_METADATA_PROGRAM_ID } from "@metaplex-foundation/mpl-token-metadata";

describe("prediction_market", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace
    .PredictionMarketAmm as Program<PredictionMarketAmm>;

  const providerWallet = provider.wallet as NodeWallet;

  let mintYes: PublicKey;
  let mintNo: PublicKey;
  let mintUSDC: PublicKey = new PublicKey(
    "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU"
  );
  let market: PublicKey;
  let vaultYes: PublicKey;
  let vaultNo: PublicKey;
  let vaultUSDC: PublicKey;
  let userAtaUSDC: PublicKey;
  let userAtaYes: PublicKey;
  let userAtaNo: PublicKey;

  const seed = new anchor.BN(
    Math.floor(Date.now() / 1000) * 1000000 + Math.floor(Math.random() * 1000000)
  );
  const marketName = "VIRAT_CENTURY_IND_NZ_CT_2025";
  const fee = 100;
  const endTime = new anchor.BN(Math.floor(Date.now() / 1000) + 86400);

  const uri_yes = "https://gateway.irys.xyz/52pWSqmBFhEr67znFS4KoK5UBpwgbJ1hHH6qQyUbkD6V";
  const uri_no = "https://gateway.irys.xyz/CPgxvKRwE6D4UVTvaWMbm3tJntvumZGpegXeALthdY5s";

  const getMetadataAddress = (mint: PublicKey): PublicKey => {
    return PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        new PublicKey(MPL_TOKEN_METADATA_PROGRAM_ID).toBuffer(),
        mint.toBuffer(),
      ],
      new PublicKey(MPL_TOKEN_METADATA_PROGRAM_ID)
    )[0];
  };

  console.log("starting test")
  it("Initialize the market and mint tokens", async () => {
    console.log("starting test now")
    const [marketPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("market"), seed.toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    market = marketPda;

    const metadata_yes = {
      name: "VIRAT_YES",
      symbol: "VK_YES",
      description: "VIRAT KOHLI YES token IND_NZ_2025_CT",
      image: "https://github.com/user-attachments/assets/9b77f718-0e8e-466a-8bb5-8c9c97eece33",
    };

    const metadata_no = {
      name: "VIRAT_NO",
      symbol: "VK_NO",
      description: "VIRAT KOHLI NO token IND_NZ_2025_CT",
      image: "https://github.com/user-attachments/assets/9b77f718-0e8e-466a-8bb5-8c9c97eece33",
    };

    const [mintYesPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("yes_mint"), seed.toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    mintYes = mintYesPda;

    const [mintNoPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("no_mint"), seed.toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    mintNo = mintNoPda;

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
    
    const metadataYesAddress = getMetadataAddress(mintYes);
    const metadataNoAddress = getMetadataAddress(mintNo);
    
    try {
      const modifyComputeUnits = anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({ 
        units: 300000
      });
      
      const tx = await program.methods
        .initialize(
          seed, 
          marketName, 
          metadata_yes.name, metadata_yes.symbol, 
          metadata_no.name, metadata_no.symbol, 
          uri_yes, uri_no, 
          fee, endTime
        )
        .accountsStrict({
          signer: providerWallet.publicKey,
          mintYes,
          mintNo,
          mintUsdc: mintUSDC,
          vaultYes,
          vaultNo,
          vaultUsdc: vaultUSDC,
          market,
          metadataYes: metadataYesAddress,
          metadataNo: metadataNoAddress,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          tokenMetadataProgram: MPL_TOKEN_METADATA_PROGRAM_ID,
        })
        .preInstructions([modifyComputeUnits])
        .rpc({ skipPreflight: true });

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
    userAtaUSDC = getAssociatedTokenAddressSync(
      mintUSDC,
      providerWallet.publicKey,
      true,
      TOKEN_PROGRAM_ID
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
          vaultNo,
          vaultUsdc: vaultUSDC,
          vaultYes,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          mintUsdc: mintUSDC,
          user: providerWallet.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([providerWallet.payer])
        .rpc();

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
    userAtaYes = getAssociatedTokenAddressSync(
      mintYes,
      providerWallet.publicKey,
      true,
      TOKEN_PROGRAM_ID
    );
  
    userAtaNo = getAssociatedTokenAddressSync(
      mintNo,
      providerWallet.publicKey,
      true,
      TOKEN_PROGRAM_ID
    );
  
    const swapTests = [
      { amount: 2_000_000, isYes: true, description: "2 USDC YES buy" },
      { amount: 2_000_000, isYes: false, description: "2 USDC NO buy" },
      { amount: 2_500_000, isYes: true, description: "2.5 USDC YES buy" },
    ];
  
    for (const test of swapTests) {
      console.log(`\nExecuting ${test.description}`);
      
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
  
        console.log(`Swap completed, tx: ${tx}`);
        console.log("After swap balances:");
        await logBalances();
      } catch (error) {
        console.error(`Error in ${test.description}:`, error);
        throw error;
      }
    }
  });
  
  async function logBalances() {
    try {
      const vaultUSDCBalance = await provider.connection.getTokenAccountBalance(vaultUSDC);
      const vaultYesBalance = await provider.connection.getTokenAccountBalance(vaultYes);
      const vaultNoBalance = await provider.connection.getTokenAccountBalance(vaultNo);
  
      console.log("Market Balances:");
      console.log(`USDC in vault: ${vaultUSDCBalance.value.uiAmount}`);
      console.log(`YES tokens in vault: ${vaultYesBalance.value.uiAmount}`);
      console.log(`NO tokens in vault: ${vaultNoBalance.value.uiAmount}`);
      
      console.log("User Balances:");
      
      try {
        const userUSDCBalance = await provider.connection.getTokenAccountBalance(userAtaUSDC);
        console.log(`USDC: ${userUSDCBalance.value.uiAmount}`);
      } catch {
        console.log("USDC: Account not found");
      }
      
      try {
        const userYesBalance = await provider.connection.getTokenAccountBalance(userAtaYes);
        console.log(`YES tokens: ${userYesBalance.value.uiAmount}`);
      } catch {
        console.log("YES tokens: Account not found");
      }
      
      try {
        const userNoBalance = await provider.connection.getTokenAccountBalance(userAtaNo);
        console.log(`NO tokens: ${userNoBalance.value.uiAmount}`);
      } catch {
        console.log("NO tokens: Account not found");
      }
    } catch (error) {
      console.error("Error in logBalances:", error);
    }
  }
});
