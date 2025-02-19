# Cricket Prediction Market

The contract allows users to participate in a decentralized prediction market by placing bets on the outcomes of cricket matches.

## Market Mechanism (LMSR)

This prediction market uses the **Logarithmic Market Scoring Rule (LMSR)**, a mathematical formula designed specifically for prediction markets. Here's how it works:

### Core Formula
The market uses LMSR to determine prices and costs:

1. **Cost Function**:  
   C = b · ln(e^(q_yes/b) + e^(q_no/b))
   
2. **YES Share Price**:  
   P_yes = e^(q_yes/b) / (e^(q_yes/b) + e^(q_no/b))

3. **NO Share Price**:  
   P_no = 1 - P_yes

Where:
- b = Liquidity parameter (controls price sensitivity)
- q_yes = Number of YES shares
- q_no = Number of NO shares

### Example: "Will India win the 2025 Champions Trophy?"

Let's walk through a trading scenario:

**Initial Market State:**
```bash
b = 1000                # Liquidity parameter
q_yes = 100_000        # Initial YES shares
q_no = 100_000         # Initial NO shares
```

1. **Initial Prices:**
   P_yes = e^(100,000/1000) / (e^(100,000/1000) + e^(100,000/1000))
   = e^100 / (e^100 + e^100)
   = 0.5 (50%)

   Both outcomes start at equal probability

2. **Someone buys 50,000 YES shares:**
   ```bash
   New q_yes = 150,000
   ```
   P_yes = e^(150,000/1000) / (e^(150,000/1000) + e^(100,000/1000))
   = e^150 / (e^150 + e^100)
   ≈ 0.73 (73%)

   Market now shows 73% chance of India winning

3. **Another trader buys 75,000 NO shares:**
   ```bash
   New q_no = 175,000
   ```
   P_yes = e^150 / (e^150 + e^175)
   ≈ 0.31 (31%)

   Market now shows 31% chance of India winning

### How It Works in Practice

1. **Trading**:
   - Buy YES tokens if you think India will win
   - Buy NO tokens if you think India won't win
   - Price automatically adjusts with each trade

2. **Settlement**:
   - When the event concludes, winning tokens are worth 1 USDC
   - Losing tokens are worth 0 USDC

3. **Example Trade**:
   - Current YES price: 0.5 USDC
   - Buy 1000 YES tokens
   - Cost = ~500 USDC (plus price impact)
   - If India wins: Receive 1000 USDC
   - If India loses: Receive 0 USDC
