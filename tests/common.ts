import { BN, web3 } from "@coral-xyz/anchor";
import path from "path";
import fs from "fs";
import { PredictionMarketAmmProgram } from "./program";
import {
  createAssociatedTokenAccount,
  createMint,
  mintTo,
} from "@solana/spl-token";
export type KP = web3.Keypair;
export type PK = web3.PublicKey;
export const SYSTEM_PROGRAM_ID = web3.SystemProgram.programId;
export class Common {
  static getUsdcMintKeypair(): KP {
    const adminJsonPath = path.join(__dirname, "..", "usdc.json");
    const adminJsonContent = fs.readFileSync(adminJsonPath, "utf8");
    const adminKeypairData = JSON.parse(adminJsonContent);
    const secretKey = Uint8Array.from(adminKeypairData);
    return web3.Keypair.fromSecretKey(secretKey);
  }
  static async createAndAirdropKeypair(
    keypair: KP = web3.Keypair.generate(),
    lamports: number = 1000000000
  ): Promise<web3.Keypair> {
    const connection = this.provider.connection;
    const airdropSignature = await connection.requestAirdrop(
      keypair.publicKey,
      lamports
    );
    await connection.confirmTransaction(airdropSignature, "confirmed");
    return keypair;
  }
  static get wallet() {
    return PredictionMarketAmmProgram.getInstance().wallet;
  }
  static get provider() {
    return PredictionMarketAmmProgram.getInstance().provider;
  }
  static get usdcMint() {
    return PredictionMarketAmmProgram.getInstance().usdcMint;
  }
  static async airdrop(address: PK, lamports: number = 1000000000) {
    const connection =
      PredictionMarketAmmProgram.getInstance().provider.connection;
    const airdropSignature = await connection.requestAirdrop(address, lamports);
    await connection.confirmTransaction(airdropSignature, "confirmed");
  }
  // static async initializeUserWithAtas(
  //   user: KP = web3.Keypair.generate(),
  //   marketId: BN,
  //   usdcAmount: number | bigint = 100_000_000
  // ) {
  //   const userUsdcAta = await this.createAta(
  //     user.publicKey,
  //     this.usdcMint.publicKey
  //   );
  //   const userYesAta = PredictionMarketAmmProgram.getAtaAddress(
  //     PredictionMarketAmmProgram.yesMintAddress(marketId),
  //     user.publicKey
  //   );
  //   const userNoAta = PredictionMarketAmmProgram.getAtaAddress(
  //     PredictionMarketAmmProgram.noMintAddress(marketId),
  //     user.publicKey
  //   );
  //   this.mint(this.usdcMint.publicKey, userUsdcAta, usdcAmount);
  //   return {
  //     keypair: user,
  //     userNoAta,
  //     userUsdcAta,
  //     userYesAta,
  //   } as UserKeys;
  // }
  static async initializeMint(mint: KP, decimals: number, mintAuthority: PK) {
    await createMint(
      this.provider.connection,
      this.wallet.payer,
      mintAuthority,
      null,
      decimals,
      mint,
      { commitment: "confirmed" }
    );
  }
  static async createAta(owner: PK, mint: PK) {
    return await createAssociatedTokenAccount(
      this.provider.connection,
      this.wallet.payer,
      mint,
      owner
    );
  }
  static async mint(
    mint: PK,
    destination: PK,
    amount: number | bigint = 100_000_000,
    authority: PK = Common.wallet.publicKey
  ) {
    await mintTo(
      this.provider.connection,
      Common.wallet.payer,
      mint,
      destination,
      authority,
      amount
    );
  }
  static async fetchTokenBalance(tokenAddress: PK) {
    const amt = (
      await this.provider.connection.getTokenAccountBalance(tokenAddress)
    ).value.amount;
    return new BN(amt);
  }
}

export interface UserKeys {
  keypair: KP;
  userNoAta: PK;
}
