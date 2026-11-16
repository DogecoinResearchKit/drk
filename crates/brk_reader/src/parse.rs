use std::io::Cursor;

use bitcoin::{Block as BitcoinBlock, Transaction, VarInt, block::Header, consensus::Decodable};
use brk_error::{Error, Result};
use brk_types::{BlkPosition, BlockHash, Height, ReadBlock};

use crate::{XORBytes, XORIndex, canonical::CanonicalRange};

pub const HEADER_LEN: usize = 80;

/// The canonical offset and header of the block in `bytes`, `None` for any other block. A block
/// of the range that does not follow its canonical predecessor is an error: the range was resolved
/// across a reorg. Decodes the header onto a stack buffer so `bytes` stays untouched: the body
/// parse later re-XORs the full block from the original phase.
pub fn peek_canonical(
    bytes: &[u8],
    mut xor_state: XORIndex,
    xor_bytes: XORBytes,
    canonical: &CanonicalRange,
) -> Result<Option<(u32, Header)>> {
    if bytes.len() < HEADER_LEN {
        return Ok(None);
    }
    let mut header_buf = [0u8; HEADER_LEN];
    header_buf.copy_from_slice(&bytes[..HEADER_LEN]);
    xor_state.bytes(&mut header_buf, xor_bytes);
    let header = Header::consensus_decode_from_finite_reader(&mut &header_buf[..])
        .expect("an 80-byte header always decodes");
    let Some(offset) = canonical.offset_of(&BlockHash::from(header.block_hash())) else {
        return Ok(None);
    };
    if !canonical.verify_prev(offset, &BlockHash::from(header.prev_blockhash)) {
        return Err(Error::Internal("canonical batch stitched across a reorg"));
    }
    Ok(Some((offset, header)))
}

pub fn parse_canonical_body(
    mut bytes: Vec<u8>,
    position: BlkPosition,
    mut xor_state: XORIndex,
    xor_bytes: XORBytes,
    height: Height,
    header: Header,
) -> Result<ReadBlock> {
    debug_assert!(bytes.len() >= HEADER_LEN);
    xor_state.bytes(&mut bytes, xor_bytes);
    let bitcoin_hash = header.block_hash();

    let mut cursor = Cursor::new(bytes);
    cursor.set_position(HEADER_LEN as u64);
    #[cfg(feature = "dogecoin")]
    brk_types::dogecoin::skip_auxpow(&header, &mut cursor)?;

    // `from_finite_reader` skips the `Take<R>` wrap that
    // `consensus_decode` applies to every nested field for memory
    // safety: our cursor is already a bounded `Vec<u8>`, so the
    // wrapping is pure overhead and compounds across ~2000 tx fields.
    let tx_count = VarInt::consensus_decode_from_finite_reader(&mut cursor)?.0 as usize;
    let mut txdata = Vec::with_capacity(tx_count);
    let mut tx_offsets = Vec::with_capacity(tx_count);
    for _ in 0..tx_count {
        tx_offsets.push(cursor.position() as u32);
        let tx = Transaction::consensus_decode_from_finite_reader(&mut cursor)?;
        txdata.push(tx);
    }

    Ok(ReadBlock::new(
        height,
        BlockHash::from(bitcoin_hash),
        BitcoinBlock { header, txdata },
        position,
        cursor.into_inner(),
        tx_offsets,
    ))
}

#[cfg(all(test, feature = "dogecoin"))]
mod dogecoin_tests {
    use bitcoin::{consensus::Decodable, dogecoin::Block as DogecoinBlock};

    use super::*;

    fn decode_hex(hex: &str) -> Vec<u8> {
        let hex = hex.trim();
        (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect()
    }

    /// Parses `hex` the way the pipeline does (80-byte header peek, then the body) and checks the
    /// result against the AuxPoW-aware `dogecoin::Block` decoder.
    fn assert_parses(hex: &str, height: u32, expect_auxpow: bool) {
        let bytes = decode_hex(hex);
        let reference = DogecoinBlock::consensus_decode(&mut &bytes[..]).expect("reference decode");
        assert_eq!(reference.header.aux_pow.is_some(), expect_auxpow);

        let header = Header::consensus_decode_from_finite_reader(&mut &bytes[..HEADER_LEN])
            .expect("pure header decode");
        let block = parse_canonical_body(
            bytes,
            BlkPosition::new(0, 0),
            XORIndex::at_offset(0),
            XORBytes::from([0u8; 8]),
            Height::new(height),
            header,
        )
        .expect("parse_canonical_body");

        assert_eq!(block.txdata.len(), reference.txdata.len());
        assert_eq!(block.hash(), &BlockHash::from(reference.block_hash()));
    }

    /// Version 1, no AuxPoW: the transaction count follows the 80-byte header.
    #[test]
    fn parses_legacy_block() {
        assert_parses(LEGACY_100_000, 100_000, false);
    }

    /// The first merge-mined block: an AuxPoW proof sits between the header and the count.
    #[test]
    fn parses_merge_mined_block() {
        assert_parses(AUXPOW_371_337, 371_337, true);
    }

    const LEGACY_100_000: &str = include_str!("../fixtures/dogecoin_100000.hex");

    const AUXPOW_371_337: &str = include_str!("../fixtures/dogecoin_371337.hex");
}
