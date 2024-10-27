import {
  AnchorProvider,
  Program,
  setProvider,
  Wallet,
  web3,
  workspace,
} from "@coral-xyz/anchor";
import { PredictionMarketAmm } from "../target/types/prediction_market_amm";
export type KP = web3.Keypair;
export type PK = web3.PublicKey;
import { Common } from "./common";
export class PredictionMarketAmmProgram {
  private static instance: PredictionMarketAmmProgram;
  private _programId: web3.PublicKey | null = null;
  private _provider: AnchorProvider | null = null;
  private _program: Program<PredictionMarketAmm> | null = null;
  private _wallet: Wallet | null = null;
  private _usdcMint: KP | null = null;
  private constructor() {}
  static getInstance(): PredictionMarketAmmProgram {
    if (!PredictionMarketAmmProgram.instance) {
      PredictionMarketAmmProgram.instance = new PredictionMarketAmmProgram();
    }
    return PredictionMarketAmmProgram.instance;
  }
  get programId(): PK {
    if (!this._program) {
      this.initializeProgram();
    }
    return this._programId!;
  }
  get provider(): AnchorProvider {
    if (!this._program || !this._provider) {
      this.initializeProgram();
    }
    return this._provider as AnchorProvider;
  }
  get program(): Program<PredictionMarketAmm> {
    if (!this._program) {
      this.initializeProgram();
    }
    return this._program!;
  }
  get wallet(): Wallet {
    if (!this._program || !this._wallet) {
      this.initializeProgram();
    }
    return this._wallet as Wallet;
  }
  get usdcMint(): KP {
    if (!this._program || !this._usdcMint) {
      this.initializeProgram();
    }
    return this._usdcMint as KP;
  }
  private initializeProgram(): void {
    if (!this._program) {
      let provider = AnchorProvider.env();
      setProvider(provider);
      const program =
        workspace.PredictionMarketAmm as Program<PredictionMarketAmm>;
      const wallet = provider.wallet as Wallet;
      const usdcMint = Common.getUsdcMintKeypair();
      this._wallet = wallet;
      this._program = program;
      this._usdcMint = usdcMint;
      this._programId = program.programId;
      this._provider = provider;
    }
  }
  static async initializeMarket() {
    try {
    } catch (error) {}
  }
}
