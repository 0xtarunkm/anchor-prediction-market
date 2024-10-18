import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { PredictionMarketAmm } from '../target/types/prediction_market_amm';
import {
  createMint,
  getAssociatedTokenAddressSync,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
} from '@solana/spl-token';
import { PublicKey, SendTransactionError } from '@solana/web3.js';
import NodeWallet from '@coral-xyz/anchor/dist/cjs/nodewallet';

describe('prediction_market_amm', () => {
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
  const marketName = 'VK_100_IND_BAN_2024';
  const fee = 100;
  const endTime = new anchor.BN(Math.floor(Date.now() / 1000) + 86400);

  it('Airdrop SOL to user', async () => {
    const tx = await provider.connection.requestAirdrop(
      providerWallet.publicKey,
      1000000000
    );
    await provider.connection.confirmTransaction(tx);
    console.log(
      'User balance:',
      await provider.connection.getBalance(providerWallet.publicKey)
    );
  });

  it('Initialize the market and mint tokens', async () => {
    const [marketPda] = PublicKey.findProgramAddressSync(
      [Buffer.from('market'), seed.toArrayLike(Buffer, 'le', 8)],
      program.programId
    );
    market = marketPda;

    const [mintLp] = PublicKey.findProgramAddressSync(
      [Buffer.from('lp'), market.toBytes()],
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

      console.log('Market initialized, transaction signature:', tx);
    } catch (error) {
      if (error instanceof SendTransactionError) {
        console.error('Transaction failed:', error.message);
        console.error('Logs:', error.logs);
      } else {
        console.error('An unexpected error occurred:', error);
      }
      throw error;
    }
  });

  it('Deposit into the market', async () => {
    userAtaUSDC = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        providerWallet.payer,
        mintUSDC,
        providerWallet.publicKey
      )
    ).address;
    console.log('user ata usdc', userAtaUSDC);

    await mintTo(
      provider.connection,
      providerWallet.payer,
      mintUSDC,
      userAtaUSDC,
      providerWallet.publicKey,
      1_000_000_000 // 1000 USDC
    );

    userAtaYes = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        providerWallet.payer,
        mintYes,
        providerWallet.publicKey
      )
    ).address;
    console.log(`User ata YES`, userAtaYes);

    userAtaNo = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        providerWallet.payer,
        mintNo,
        providerWallet.publicKey
      )
    ).address;
    console.log(`User ata no`, userAtaNo);

    try {
      const tx = await program.methods
        .addLiquidity(
          new anchor.BN(100000000), // 100 USDC
          new anchor.BN(10000000), // 10 YES tokens
          new anchor.BN(10000000), // 10 NO tokens
          new anchor.BN(Math.floor(Date.now() / 1000) + 3600) // Deadline in 1 hour
        )
        .accountsPartial({
          market,
          mintNo,
          mintYes,
          userAtaLp: userAtaLP,
          userAtaNo,
          userAtaUsdc: userAtaUSDC,
          userAtaYes,
          vaultNo,
          vaultUsdc: vaultUSDC,
          vaultYes,
          mintLp: mintLP,
          mintUsdc: mintUSDC,
          user: providerWallet.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([providerWallet.payer])
        .rpc({ skipPreflight: true });

      const initialUserUSDCBalance =
        await provider.connection.getTokenAccountBalance(userAtaUSDC);
      const initialUserYesBalance =
        await provider.connection.getTokenAccountBalance(userAtaYes);
      const initialUserNoBalance =
        await provider.connection.getTokenAccountBalance(userAtaNo);

      console.log(
        `intial USDC ${initialUserUSDCBalance.value.amount} initial YES balance ${initialUserYesBalance.value.amount} initial NO balance ${initialUserNoBalance.value.amount}`
      );

      console.log('Liquidity added, transaction signature:', tx);
    } catch (error) {
      if (error instanceof SendTransactionError) {
        console.error('Transaction failed:', error.message);
        console.error('Logs:', error.logs);
      } else {
        console.error('An unexpected error occurred:', error);
      }
      throw error;
    }
  });

  it('Swap USDC for YES tokens', async () => {
    await mintTo(
      provider.connection,
      providerWallet.payer,
      mintUSDC,
      userAtaUSDC,
      providerWallet.publicKey,
      100_000_000
    );

    const initialUserUSDCBalance =
      await provider.connection.getTokenAccountBalance(userAtaUSDC);
    const initialUserYesBalance =
      await provider.connection.getTokenAccountBalance(userAtaYes);
    const initialUserNoBalance =
      await provider.connection.getTokenAccountBalance(userAtaNo);

    console.log(
      `intial USDC ${initialUserUSDCBalance.value} initial YES balance ${initialUserYesBalance.value} initial NO balance ${initialUserNoBalance.value}`
    );

    const amountIn = new anchor.BN(10_000_000);
    const minOut = new anchor.BN(1_000_000);
    const expiration = new anchor.BN(Math.floor(Date.now() / 1000) + 60);

    try {
      const tx = await program.methods
        .swap(true, amountIn, true, minOut, expiration)
        .accountsPartial({
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
        })
        .signers([providerWallet.payer])
        .rpc();

      console.log('Swap completed, transaction signature:', tx);
    } catch (error) {
      if (error instanceof SendTransactionError) {
        console.error('Transaction failed:', error.message);
        console.error('Logs:', error.logs);
      } else {
        console.error('An unexpected error occurred:', error);
      }
      throw error;
    }
  });

  it('Swap USDC for NO tokens', async () => {
    await mintTo(
      provider.connection,
      providerWallet.payer,
      mintUSDC,
      userAtaUSDC,
      providerWallet.publicKey,
      100_000_000
    );

    const initialUserUSDCBalance =
      await provider.connection.getTokenAccountBalance(userAtaUSDC);
    const initialUserYesBalance =
      await provider.connection.getTokenAccountBalance(userAtaYes);
    const initialUserNoBalance =
      await provider.connection.getTokenAccountBalance(userAtaNo);

    console.log(
      `intial USDC ${initialUserUSDCBalance.value.amount} initial YES balance ${initialUserYesBalance.value.amount} initial NO balance ${initialUserNoBalance.value.amount}`
    );

    const amountIn = new anchor.BN(10_000_000);
    const minOut = new anchor.BN(1_000_000);
    const expiration = new anchor.BN(Math.floor(Date.now() / 1000) + 60);

    try {
      const tx = await program.methods
        .swap(true, amountIn, false, minOut, expiration)
        .accountsPartial({
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
        })
        .signers([providerWallet.payer])
        .rpc();

      console.log('Swap completed, transaction signature:', tx);
    } catch (error) {
      if (error instanceof SendTransactionError) {
        console.error('Transaction failed:', error.message);
        console.error('Logs:', error.logs);
      } else {
        console.error('An unexpected error occurred:', error);
      }
      throw error;
    }
  });
});
