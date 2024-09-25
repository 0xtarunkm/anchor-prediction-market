import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { PredictionMarketAmm } from '../target/types/prediction_market_amm';
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createMint,
  getAssociatedTokenAddressSync,
  TOKEN_PROGRAM_ID,
} from '@solana/spl-token';
import { PublicKey, SystemProgram } from '@solana/web3.js';
import NodeWallet from '@coral-xyz/anchor/dist/cjs/nodewallet';

describe('prediction_market_amm', () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  let providerWallet = provider.wallet as NodeWallet;

  const program = anchor.workspace
    .PredictionMarketAmm as Program<PredictionMarketAmm>;

  let mintYes: PublicKey;
  let mintNo: PublicKey;
  let mintStablecoin: PublicKey;
  let auth: PublicKey;
  let market: PublicKey;
  let vaultYes: PublicKey;
  let vaultNo: PublicKey;
  let vaultStablecoin: PublicKey;

  const seed = new anchor.BN(Math.floor(Math.random() * 1000000));
  const marketName = 'Test Market';
  const fee = 100;
  const endTime = new anchor.BN(Math.floor(Date.now() / 1000) + 86400);

  it('Is initialized!', async () => {
    const [authPda] = PublicKey.findProgramAddressSync(
      [Buffer.from('auth')],
      program.programId
    );
    auth = authPda;

    const [marketPda] = PublicKey.findProgramAddressSync(
      [Buffer.from('market'), seed.toArrayLike(Buffer, 'le', 8)],
      program.programId
    );
    market = marketPda;

    // create mints
    mintYes = await createMint(
      provider.connection,
      providerWallet.payer,
      providerWallet.publicKey,
      null,
      6
    );
    mintNo = await createMint(
      provider.connection,
      providerWallet.payer,
      providerWallet.publicKey,
      null,
      6
    );
    mintStablecoin = await createMint(
      provider.connection,
      providerWallet.payer,
      providerWallet.publicKey,
      null,
      6
    );

    // Get associated token accounts
    vaultYes = getAssociatedTokenAddressSync(mintYes, auth, true);
    vaultNo = getAssociatedTokenAddressSync(mintNo, auth, true);
    vaultStablecoin = getAssociatedTokenAddressSync(mintStablecoin, auth, true);

    await program.methods
      .initialize(seed, marketName, fee, endTime, null)
      .accountsPartial({
        signer: providerWallet.publicKey,
        mintYes,
        mintNo,
        mintStablecoin,
        vaultYes,
        vaultNo,
        vaultStablecoin,
        market,
        auth,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      });
  });
});
