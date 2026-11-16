use std::io::Read;

use crate::{ReadPin, check_bound};
#[cfg(not(feature = "dogecoin"))]
use bitcoin::Block as BitcoinBlock;
use bitcoin::{Weight, consensus::deserialize};
use brk_reader::{BlkRead, MAGIC_BYTES};
use brk_types::{BlkPosition, BlockHash, Height};
use vecdb::ReadableVec;

use crate::{Error, OptionData, Query, Result};

// Serialized bytes cannot exceed weight: each byte contributes at least one WU.
const MAX_BLOCK_BYTES: u64 = Weight::MAX_BLOCK.to_wu();

struct RawRecord {
    reader: BlkRead,
    size: u64,
    weight: u64,
    count: u32,
}

impl Query {
    fn open_raw_record(&self, height: Height, guard: &ReadPin) -> Result<RawRecord> {
        let safe = guard.lengths();
        check_bound(guard, height, safe.height)?;
        if height >= safe.height {
            return Err(Error::NotFound("Block not found".into()));
        }
        let vecs = self.indexer().vecs();
        let position = vecs.blocks.position.collect_one(height).data()?;
        let size = u64::from(*vecs.blocks.size.collect_one(height).data()?);
        let weight = u64::from(*vecs.blocks.weight.collect_one(height).data()?);
        if weight > Weight::MAX_BLOCK.to_wu() || size > weight {
            return Err(Error::Internal("Invalid indexed block weight"));
        }
        let (_, count) = self.block_tx_range(height, guard)?;
        let offset = position
            .offset()
            .checked_sub(8)
            .ok_or(Error::Internal("Block position precedes its frame"))?;
        let reader = self
            .reader()
            .reader_at(BlkPosition::new(position.blk_index(), offset))?;
        Ok(RawRecord {
            reader,
            size,
            weight,
            count: count as u32,
        })
    }

    fn read_raw_record(mut reader: impl Read, size: u64, hash: &BlockHash) -> Result<Vec<u8>> {
        let header = Self::read_raw_prefix(&mut reader, size, hash)?;
        let mut bytes = vec![0; size as usize];
        bytes[..80].copy_from_slice(&header);
        reader.read_exact(&mut bytes[80..])?;
        Ok(bytes)
    }

    fn read_raw_prefix(mut reader: impl Read, size: u64, hash: &BlockHash) -> Result<[u8; 80]> {
        if !(81..=MAX_BLOCK_BYTES).contains(&size) {
            return Err(Error::Internal("Invalid indexed block size"));
        }
        // This reader indexes mainnet blk records (same magic as scan).
        // Read framing and header together, before allocating the payload.
        let mut prefix = [0u8; 88];
        reader.read_exact(&mut prefix)?;
        if prefix[..4] != MAGIC_BYTES
            || u64::from(u32::from_le_bytes(prefix[4..8].try_into().unwrap())) != size
        {
            return Err(Error::Internal("Block frame differs from index"));
        }
        Self::verify_header(&prefix[8..], hash)?;
        Ok(prefix[8..].try_into().unwrap())
    }

    #[cfg(not(feature = "dogecoin"))]
    fn verify_raw_payload(bytes: &[u8], weight: u64, count: u32) -> Result<()> {
        // Check the indexed count before a decoder allocates its transaction list.
        Self::read_block_tx_count(&bytes[80..], count)?;
        let block: BitcoinBlock =
            deserialize(bytes).map_err(|_| Error::Internal("Invalid raw block payload"))?;
        if block.weight().to_wu() != weight
            || !block.check_merkle_root()
            || !block.check_witness_commitment()
        {
            return Err(Error::Internal("Raw block commitments differ from index"));
        }
        Ok(())
    }

    /// Dogecoin has no witness data, so a block weighs four times its size. The transaction count
    /// follows the AuxPoW of a merge-mined block, not the header.
    #[cfg(feature = "dogecoin")]
    fn verify_raw_payload(bytes: &[u8], weight: u64, count: u32) -> Result<()> {
        use bitcoin::{block::Header, consensus::Decodable};
        let mut cursor = std::io::Cursor::new(bytes);
        let header = Header::consensus_decode_from_finite_reader(&mut cursor)
            .map_err(|_| Error::Internal("Invalid raw block payload"))?;
        brk_types::dogecoin::skip_auxpow(&header, &mut cursor)
            .map_err(|_| Error::Internal("Invalid raw block payload"))?;
        Self::read_block_tx_count(&bytes[cursor.position() as usize..], count)?;
        let block: bitcoin::dogecoin::Block =
            deserialize(bytes).map_err(|_| Error::Internal("Invalid raw block payload"))?;
        if 4 * bytes.len() as u64 != weight || !block.check_merkle_root() {
            return Err(Error::Internal("Raw block commitments differ from index"));
        }
        Ok(())
    }

    /// Borrow the pinned immutable prefix through the complete payload read.
    pub(super) fn block_raw_at_height(
        &self,
        height: Height,
        hash: &BlockHash,
        guard: &ReadPin,
    ) -> Result<Vec<u8>> {
        let mut record = self.open_raw_record(height, guard)?;
        let bytes = Self::read_raw_record(&mut record.reader, record.size, hash)?;
        Self::verify_raw_payload(&bytes, record.weight, record.count)?;
        Ok(bytes)
    }
    /// Validate framing and identity without reading or allocating the payload.
    /// Borrows the pinned prefix; payload integrity remains a GET check.
    pub(super) fn block_raw_size_at_height(
        &self,
        height: Height,
        hash: &BlockHash,
        guard: &ReadPin,
    ) -> Result<u64> {
        let mut record = self.open_raw_record(height, guard)?;
        Self::read_raw_prefix(&mut record.reader, record.size, hash)?;
        Ok(record.size)
    }
}

#[cfg(all(test, feature = "dogecoin"))]
mod dogecoin_tests {
    use super::*;

    fn block(hex: &str) -> Vec<u8> {
        let hex = hex.trim();
        (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect()
    }

    fn tx_count(bytes: &[u8]) -> u32 {
        deserialize::<bitcoin::dogecoin::Block>(bytes).unwrap().txdata.len() as u32
    }

    fn verify(hex: &str) -> Result<()> {
        let bytes = block(hex);
        Query::verify_raw_payload(&bytes, 4 * bytes.len() as u64, tx_count(&bytes))
    }

    #[test]
    fn verifies_legacy_and_merge_mined_blocks() {
        verify(include_str!("../../../../brk_reader/fixtures/dogecoin_28092.hex")).unwrap();
        verify(include_str!("../../../../brk_reader/fixtures/dogecoin_372992.hex")).unwrap();
    }

    #[test]
    fn rejects_a_wrong_count_or_weight() {
        let hex = include_str!("../../../../brk_reader/fixtures/dogecoin_372992.hex");
        let bytes = block(hex);
        let (weight, count) = (4 * bytes.len() as u64, tx_count(&bytes));
        assert!(Query::verify_raw_payload(&bytes, weight, count + 1).is_err());
        assert!(Query::verify_raw_payload(&bytes, weight / 4, count).is_err());
    }
}
