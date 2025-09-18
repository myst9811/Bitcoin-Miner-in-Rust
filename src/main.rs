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
}

impl Block {
    pub fn new(
        version: u32,
        previous_hash: [u8; 32],
        transactions: Vec<Transaction>,
        difficulty_target: u32,
    ) -> Self {
        let merkle_root = Self::calculate_merkle_root(&transactions);
        let header = BlockHeader::new(version, previous_hash, merkle_root, difficulty_target);

        Self {
            header,
            transactions,
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
        writeln!(f, "Block {{")?;
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

// Mining module
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
    ) -> Result<Block, MiningError> {
        let mut block = Block::new(version, previous_hash, transactions, self.difficulty_target);

        println!(
            "Starting mining with difficulty target: {}",
            self.difficulty_target
        );
        println!("Previous hash: {}", hex::encode(previous_hash));
        println!("Merkle root: {}", hex::encode(block.header.merkle_root));
        println!();

        let start_time = SystemTime::now();

        loop {
            let hash = block.hash();

            if self.meets_difficulty_target(&hash) {
                let mining_time = start_time.elapsed().unwrap();
                println!(
                    "✅ Block mined successfully! Nonce: {}, Time: {:.2}s",
                    block.header.nonce,
                    mining_time.as_secs_f64()
                );
                println!("🎯 Block hash: {}", hex::encode(hash));
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

            // Print progress every million attempts
            if block.header.nonce % 1_000_000 == 0 {
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
                println!("Difficulty decreased to: {}", self.difficulty_target);
            }
        } else if ratio < 0.5 {
            // Too fast, increase difficulty
            self.difficulty_target += 1;
            println!("Difficulty increased to: {}", self.difficulty_target);
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

    Block::new(1, [0; 32], vec![genesis_transaction], 4)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_hash() {
        let tx = Transaction::new(
            "test_tx".to_string(),
            vec!["input1".to_string()],
            vec!["output1".to_string()],
            100,
        );
        let hash = tx.hash();
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_block_creation() {
        let tx = Transaction::new(
            "test_tx".to_string(),
            vec!["input1".to_string()],
            vec!["output1".to_string()],
            100,
        );
        let block = Block::new(1, [0; 32], vec![tx], 4);
        assert_eq!(block.header.version, 1);
        assert_eq!(block.transactions.len(), 1);
    }

    #[test]
    fn test_mining() {
        let miner = Miner::new(4); // Low difficulty for testing
        let tx = Transaction::new(
            "test_tx".to_string(),
            vec!["input1".to_string()],
            vec!["output1".to_string()],
            100,
        );

        let result = miner.mine_block(1, [0; 32], vec![tx]);
        assert!(result.is_ok());

        let block = result.unwrap();
        assert!(miner.validate_block(&block));
    }

    #[test]
    fn test_difficulty_validation() {
        let miner = Miner::new(8);
        let hash_easy = [
            0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
        ]; // 9 leading zeros
        let hash_hard = [
            128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
        ]; // 0 leading zeros

        assert!(miner.meets_difficulty_target(&hash_easy));
        assert!(!miner.meets_difficulty_target(&hash_hard));
    }
}

// Example usage
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a miner with difficulty target of 12 leading zero bits
    let miner = Miner::new(12);

    // Create some sample transactions
    let tx1 = Transaction::new(
        "tx1".to_string(),
        vec!["input1".to_string()],
        vec!["1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2".to_string()],
        25_00000000, // 25 BTC
    );

    let tx2 = Transaction::new(
        "tx2".to_string(),
        vec!["input2".to_string()],
        vec!["3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy".to_string()],
        15_00000000, // 15 BTC
    );

    // Mine a block
    println!("🚀 Starting Bitcoin Mining...");
    println!("=====================================");
    let block = miner.mine_block(1, [0; 32], vec![tx1, tx2])?;

    // Validate the mined block
    println!("\n🔍 Validating block...");
    if miner.validate_block(&block) {
        println!("✅ Block is valid!");
        println!("\n📦 Block Details:");
        println!("=====================================");
        println!("{}", block);
    } else {
        println!("❌ Block validation failed!");
    }

    Ok(())
}
