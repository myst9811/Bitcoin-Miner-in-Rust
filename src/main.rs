use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

// Transaction structure (simplified)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub amount: u64,
}

impl Transaction {
    pub fn new(id: String, inputs: Vec<String>, outputs: Vec<String>, amount: u64) -> Self {
        Self {
            id,
            inputs,
            outputs,
            amount,
        }
    }

    pub fn hash(&self) -> [u8; 32] {
        let serialized = serde_json::to_string(self).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(serialized.as_bytes());
        hasher.finalize().into()
    }
}

// Custom Display for Transaction
impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Transaction {{ id: {}, amount: {} satoshis }}",
            self.id, self.amount
        )
    }
}

// Block header structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub version: u32,
    pub previous_hash: [u8; 32],
    pub merkle_root: [u8; 32],
    pub timestamp: u64,
    pub difficulty_target: u32,
    pub nonce: u32,
}

impl BlockHeader {
    pub fn new(
        version: u32,
        previous_hash: [u8; 32],
        merkle_root: [u8; 32],
        difficulty_target: u32,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            version,
            previous_hash,
            merkle_root,
            timestamp,
            difficulty_target,
            nonce: 0,
        }
    }

    pub fn hash(&self) -> [u8; 32] {
        let serialized = serde_json::to_string(self).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(serialized.as_bytes());
        let first_hash = hasher.finalize();

        // Double SHA-256 (Bitcoin standard)
        let mut hasher = Sha256::new();
        hasher.update(first_hash);
        hasher.finalize().into()
    }
}

// Custom Display for BlockHeader
impl fmt::Display for BlockHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "BlockHeader {{")?;
        writeln!(f, "  version: {}", self.version)?;
        writeln!(f, "  previous_hash: {}", hex::encode(self.previous_hash))?;
        writeln!(f, "  merkle_root: {}", hex::encode(self.merkle_root))?;
        writeln!(f, "  timestamp: {}", self.timestamp)?;
        writeln!(f, "  difficulty_target: {}", self.difficulty_target)?;
        writeln!(f, "  nonce: {}", self.nonce)?;
        write!(f, "}}")
    }
}

// Block structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
    pub height: u32,
}

impl Block {
    pub fn new(
        version: u32,
        previous_hash: [u8; 32],
        transactions: Vec<Transaction>,
        difficulty_target: u32,
        height: u32,
    ) -> Self {
        let merkle_root = Self::calculate_merkle_root(&transactions);
        let header = BlockHeader::new(version, previous_hash, merkle_root, difficulty_target);

        Self {
            header,
            transactions,
            height,
        }
    }

    pub fn hash(&self) -> [u8; 32] {
        self.header.hash()
    }

    // Simplified merkle root calculation
    fn calculate_merkle_root(transactions: &[Transaction]) -> [u8; 32] {
        if transactions.is_empty() {
            return [0; 32];
        }

        let mut hashes: Vec<[u8; 32]> = transactions.iter().map(|tx| tx.hash()).collect();

        while hashes.len() > 1 {
            let mut next_level = Vec::new();

            for chunk in hashes.chunks(2) {
                let mut hasher = Sha256::new();
                hasher.update(&chunk[0]);

                if chunk.len() == 2 {
                    hasher.update(&chunk[1]);
                } else {
                    // If odd number, duplicate the last hash
                    hasher.update(&chunk[0]);
                }

                next_level.push(hasher.finalize().into());
            }

            hashes = next_level;
        }

        hashes[0]
    }
}

// Custom Display for Block
impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Block #{} {{", self.height)?;
        writeln!(f, "  header: {}", self.header)?;
        writeln!(f, "  block_hash: {}", hex::encode(self.hash()))?;
        writeln!(f, "  transactions: [")?;
        for (i, tx) in self.transactions.iter().enumerate() {
            writeln!(f, "    {}: {}", i, tx)?;
        }
        writeln!(f, "  ]")?;
        write!(f, "}}")
    }
}

// Blockchain structure
#[derive(Debug)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub miner: Miner,
}

impl Blockchain {
    pub fn new(difficulty: u32) -> Self {
        let mut blockchain = Self {
            blocks: Vec::new(),
            pending_transactions: Vec::new(),
            miner: Miner::new(difficulty),
        };

        // Create and add genesis block
        let genesis = genesis_block();
        blockchain.blocks.push(genesis);

        blockchain
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.pending_transactions.push(transaction);
        println!(
            "➕ Added transaction: {}",
            self.pending_transactions.last().unwrap()
        );
    }

    pub fn mine_pending_transactions(&mut self) -> Result<(), MiningError> {
        if self.pending_transactions.is_empty() {
            println!("⚠️  No pending transactions to mine");
            return Ok(());
        }

        let previous_block = self.blocks.last().unwrap();
        let previous_hash = previous_block.hash();
        let height = (self.blocks.len()) as u32;

        println!(
            "\n⛏️  Mining block #{} with {} transactions...",
            height,
            self.pending_transactions.len()
        );

        let transactions = std::mem::take(&mut self.pending_transactions);
        let new_block = self
            .miner
            .mine_block(1, previous_hash, transactions, height)?;

        self.blocks.push(new_block);
        println!("✅ Block #{} added to blockchain!", height);

        Ok(())
    }

    pub fn get_latest_block(&self) -> &Block {
        self.blocks.last().unwrap()
    }

    pub fn validate_chain(&self) -> bool {
        for i in 1..self.blocks.len() {
            let current_block = &self.blocks[i];
            let previous_block = &self.blocks[i - 1];

            // Validate block hash
            if !self.miner.validate_block(current_block) {
                println!("❌ Block #{} has invalid hash", current_block.height);
                return false;
            }

            // Validate chain linkage
            if current_block.header.previous_hash != previous_block.hash() {
                println!(
                    "❌ Block #{} has invalid previous hash",
                    current_block.height
                );
                return false;
            }
        }

        println!("✅ Blockchain is valid!");
        true
    }

    pub fn print_chain(&self) {
        println!("\n📊 BLOCKCHAIN STATUS");
        println!("=====================================");
        println!("Total blocks: {}", self.blocks.len());
        println!(
            "Total pending transactions: {}",
            self.pending_transactions.len()
        );
        println!("Current difficulty: {}", self.miner.difficulty_target);

        for block in &self.blocks {
            println!("\n{}", block);
        }
    }

    pub fn get_balance(&self, address: &str) -> u64 {
        let mut balance = 0u64;

        for block in &self.blocks {
            for transaction in &block.transactions {
                // Add to balance if address is in outputs
                if transaction.outputs.contains(&address.to_string()) {
                    balance += transaction.amount;
                }
                // Subtract from balance if address is in inputs (simplified)
                if transaction.inputs.contains(&address.to_string()) {
                    balance = balance.saturating_sub(transaction.amount);
                }
            }
        }

        balance
    }
}

// Mining module
#[derive(Debug)]
pub struct Miner {
    pub difficulty_target: u32,
}

impl Miner {
    pub fn new(difficulty_target: u32) -> Self {
        Self { difficulty_target }
    }

    pub fn mine_block(
        &self,
        version: u32,
        previous_hash: [u8; 32],
        transactions: Vec<Transaction>,
        height: u32,
    ) -> Result<Block, MiningError> {
        let mut block = Block::new(
            version,
            previous_hash,
            transactions,
            self.difficulty_target,
            height,
        );

        println!(
            "🎯 Target difficulty: {} leading zero bits",
            self.difficulty_target
        );
        println!("🔗 Previous hash: {}", hex::encode(previous_hash));
        println!("🌳 Merkle root: {}", hex::encode(block.header.merkle_root));

        let start_time = SystemTime::now();

        loop {
            let hash = block.hash();

            if self.meets_difficulty_target(&hash) {
                let mining_time = start_time.elapsed().unwrap();
                println!(
                    "⚡ Block mined! Nonce: {}, Time: {:.2}s",
                    block.header.nonce,
                    mining_time.as_secs_f64()
                );
                println!("🏆 Block hash: {}", hex::encode(hash));
                return Ok(block);
            }

            // Increment nonce and try again
            if block.header.nonce == u32::MAX {
                // If nonce overflows, update timestamp and reset nonce
                block.header.timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                block.header.nonce = 0;
            } else {
                block.header.nonce += 1;
            }

            // Print progress every 100k attempts for better visibility
            if block.header.nonce % 100_000 == 0 && block.header.nonce > 0 {
                println!("⛏️  Mining... Tried {} nonces", block.header.nonce);
            }
        }
    }

    fn meets_difficulty_target(&self, hash: &[u8; 32]) -> bool {
        // Check if hash has the required number of leading zeros
        let leading_zeros = self.count_leading_zero_bits(hash);
        leading_zeros >= self.difficulty_target
    }

    fn count_leading_zero_bits(&self, hash: &[u8; 32]) -> u32 {
        let mut count = 0;
        for byte in hash.iter() {
            if *byte == 0 {
                count += 8;
            } else {
                count += byte.leading_zeros();
                break;
            }
        }
        count
    }

    pub fn validate_block(&self, block: &Block) -> bool {
        let hash = block.hash();
        self.meets_difficulty_target(&hash)
    }

    // Adjust difficulty based on mining time
    pub fn adjust_difficulty(&mut self, actual_time: u64, target_time: u64) {
        let ratio = actual_time as f64 / target_time as f64;

        if ratio > 2.0 {
            // Too slow, decrease difficulty
            if self.difficulty_target > 1 {
                self.difficulty_target -= 1;
                println!("📉 Difficulty decreased to: {}", self.difficulty_target);
            }
        } else if ratio < 0.5 {
            // Too fast, increase difficulty
            self.difficulty_target += 1;
            println!("📈 Difficulty increased to: {}", self.difficulty_target);
        }
    }
}

#[derive(Debug)]
pub enum MiningError {
    InvalidBlock,
    MiningFailed,
}

impl std::fmt::Display for MiningError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MiningError::InvalidBlock => write!(f, "Invalid block"),
            MiningError::MiningFailed => write!(f, "Mining failed"),
        }
    }
}

impl std::error::Error for MiningError {}

// Helper functions
pub fn genesis_block() -> Block {
    let genesis_transaction = Transaction::new(
        "genesis".to_string(),
        vec![],
        vec!["1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_string()],
        50_00000000, // 50 BTC in satoshis
    );

    Block::new(1, [0; 32], vec![genesis_transaction], 8, 0) // Genesis block is height 0
}

// Sample transaction generator
pub fn create_sample_transactions() -> Vec<Transaction> {
    vec![
        Transaction::new(
            "tx_001".to_string(),
            vec!["1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_string()],
            vec!["1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2".to_string()],
            10_00000000, // 10 BTC
        ),
        Transaction::new(
            "tx_002".to_string(),
            vec!["1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_string()],
            vec!["3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy".to_string()],
            5_00000000, // 5 BTC
        ),
        Transaction::new(
            "tx_003".to_string(),
            vec!["1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2".to_string()],
            vec!["1JfbZRwdDHKZmuiZgYArJZhcuuzuw2HuMu".to_string()],
            3_50000000, // 3.5 BTC
        ),
        Transaction::new(
            "tx_004".to_string(),
            vec!["3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy".to_string()],
            vec!["1dice8EMZmqKvrGE4Qc9bUFf9PX3xaYDp".to_string()],
            2_25000000, // 2.25 BTC
        ),
        Transaction::new(
            "tx_005".to_string(),
            vec!["1JfbZRwdDHKZmuiZgYArJZhcuuzuw2HuMu".to_string()],
            vec!["1BoatSLRHtKNngkdXEeobR76b53LETtpyT".to_string()],
            1_00000000, // 1 BTC
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blockchain_creation() {
        let blockchain = Blockchain::new(4);
        assert_eq!(blockchain.blocks.len(), 1); // Genesis block
        assert_eq!(blockchain.blocks[0].height, 0);
    }

    #[test]
    fn test_transaction_addition() {
        let mut blockchain = Blockchain::new(4);
        let tx = Transaction::new(
            "test_tx".to_string(),
            vec!["input1".to_string()],
            vec!["output1".to_string()],
            100,
        );

        blockchain.add_transaction(tx);
        assert_eq!(blockchain.pending_transactions.len(), 1);
    }

    #[test]
    fn test_mining_multiple_blocks() {
        let mut blockchain = Blockchain::new(4);

        let tx1 = Transaction::new("tx1".to_string(), vec![], vec!["addr1".to_string()], 100);
        let tx2 = Transaction::new("tx2".to_string(), vec![], vec!["addr2".to_string()], 200);

        blockchain.add_transaction(tx1);
        blockchain.add_transaction(tx2);

        let result = blockchain.mine_pending_transactions();
        assert!(result.is_ok());
        assert_eq!(blockchain.blocks.len(), 2); // Genesis + 1 new block
    }

    #[test]
    fn test_blockchain_validation() {
        let mut blockchain = Blockchain::new(4);
        let tx = Transaction::new("tx1".to_string(), vec![], vec!["addr1".to_string()], 100);

        blockchain.add_transaction(tx);
        blockchain.mine_pending_transactions().unwrap();

        assert!(blockchain.validate_chain());
    }
}

// Example usage
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 BITCOIN BLOCKCHAIN MINING SIMULATOR");
    println!("=====================================\n");

    // Create a new blockchain with moderate difficulty
    let mut blockchain = Blockchain::new(8);

    println!("📋 Genesis block created successfully!");

    // Create and add multiple transactions for the first block
    println!("\n💳 Adding transactions to Block #1...");
    let transactions_block1 = vec![
        Transaction::new(
            "tx_001".to_string(),
            vec!["1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_string()],
            vec!["1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2".to_string()],
            25_00000000, // 25 BTC
        ),
        Transaction::new(
            "tx_002".to_string(),
            vec!["1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_string()],
            vec!["3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy".to_string()],
            15_00000000, // 15 BTC
        ),
        Transaction::new(
            "tx_003".to_string(),
            vec!["1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_string()],
            vec!["1JfbZRwdDHKZmuiZgYArJZhcuuzuw2HuMu".to_string()],
            8_50000000, // 8.5 BTC
        ),
    ];

    for tx in transactions_block1 {
        blockchain.add_transaction(tx);
    }

    // Mine the first block
    blockchain.mine_pending_transactions()?;

    // Add transactions for the second block
    println!("\n💳 Adding transactions to Block #2...");
    let transactions_block2 = vec![
        Transaction::new(
            "tx_004".to_string(),
            vec!["1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2".to_string()],
            vec!["1dice8EMZmqKvrGE4Qc9bUFf9PX3xaYDp".to_string()],
            12_00000000, // 12 BTC
        ),
        Transaction::new(
            "tx_005".to_string(),
            vec!["3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy".to_string()],
            vec!["1BoatSLRHtKNngkdXEeobR76b53LETtpyT".to_string()],
            7_25000000, // 7.25 BTC
        ),
    ];

    for tx in transactions_block2 {
        blockchain.add_transaction(tx);
    }

    // Mine the second block
    blockchain.mine_pending_transactions()?;

    // Add transactions for the third block
    println!("\n💳 Adding transactions to Block #3...");
    let transactions_block3 = vec![
        Transaction::new(
            "tx_006".to_string(),
            vec!["1JfbZRwdDHKZmuiZgYArJZhcuuzuw2HuMu".to_string()],
            vec!["1F1tAaz5x1HUXrCNLbtMDqcw6o5GNn4xqX".to_string()],
            4_00000000, // 4 BTC
        ),
        Transaction::new(
            "tx_007".to_string(),
            vec!["1dice8EMZmqKvrGE4Qc9bUFf9PX3xaYDp".to_string()],
            vec!["1Eqk7vVT3w5GF8mJVWK5vJfYahb4D5nVZ7".to_string()],
            6_50000000, // 6.5 BTC
        ),
        Transaction::new(
            "tx_008".to_string(),
            vec!["1BoatSLRHtKNngkdXEeobR76b53LETtpyT".to_string()],
            vec!["1HLoD9E4SDFFPDiYfNYnkBLQ85Y51J3Zb1".to_string()],
            3_75000000, // 3.75 BTC
        ),
    ];

    for tx in transactions_block3 {
        blockchain.add_transaction(tx);
    }

    // Mine the third block
    blockchain.mine_pending_transactions()?;

    // Validate the entire blockchain
    println!("\n🔍 Validating blockchain...");
    blockchain.validate_chain();

    // Print the complete blockchain
    blockchain.print_chain();

    // Show some balance information
    println!("\n💰 WALLET BALANCES");
    println!("=====================================");
    let addresses = vec![
        "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa", // Genesis
        "1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2",
        "3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy",
        "1JfbZRwdDHKZmuiZgYArJZhcuuzuw2HuMu",
        "1dice8EMZmqKvrGE4Qc9bUFf9PX3xaYDp",
    ];

    for addr in addresses {
        let balance = blockchain.get_balance(addr);
        println!(
            "{}: {} satoshis ({:.8} BTC)",
            addr,
            balance,
            balance as f64 / 100_000_000.0
        );
    }

    println!("\n🎉 Mining simulation completed successfully!");

    Ok(())
}
