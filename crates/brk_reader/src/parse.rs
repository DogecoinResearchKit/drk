use brk_chain::primitives as bitcoin;
use std::io::Cursor;

use bitcoin::{Transaction, VarInt, block::Header, consensus::Decodable};
use brk_error::{Error, Result};
use brk_types::{BlkMetadata, Block, BlockHash, Height, ReadBlock};

use crate::{XORBytes, XORIndex, canonical::CanonicalRange};

pub(crate) const HEADER_LEN: usize = 80;

/// Decodes the header onto a stack buffer so `bytes` stays untouched:
/// the body parse later re-XORs the full block from the original phase.
pub(crate) fn peek_canonical(
    bytes: &[u8],
    mut xor_state: XORIndex,
    xor_bytes: XORBytes,
    canonical: &CanonicalRange,
) -> Option<(u32, Header)> {
    if bytes.len() < HEADER_LEN {
        return None;
    }
    let mut header_buf = [0u8; HEADER_LEN];
    header_buf.copy_from_slice(&bytes[..HEADER_LEN]);
    xor_state.bytes(&mut header_buf, xor_bytes);
    let header = Header::consensus_decode_from_finite_reader(&mut &header_buf[..]).ok()?;
    let offset = canonical.offset_of(&BlockHash::from(header.block_hash()))?;
    Some((offset, header))
}

pub(crate) fn parse_canonical_body(
    mut bytes: Vec<u8>,
    metadata: BlkMetadata,
    mut xor_state: XORIndex,
    xor_bytes: XORBytes,
    height: Height,
    header: Header,
) -> Result<ReadBlock> {
    if bytes.len() < HEADER_LEN {
        return Err(Error::Internal("Block bytes shorter than header"));
    }

    xor_state.bytes(&mut bytes, xor_bytes);
    let bitcoin_hash = header.block_hash();

    let mut cursor = Cursor::new(bytes);
    // Bitcoin/Litecoin headers are a fixed 80 bytes, so the tx-count varint
    // sits immediately after. Dogecoin merge-mined blocks (height >= 371,337)
    // splice a variable-length `AuxPow` payload between the 80-byte header and
    // the varint, so we must decode the full `dogecoin::Header` to advance the
    // cursor past it. `has_auxpow_bit()` gates the AuxPoW read internally, so
    // legacy pre-fork blocks are handled by the same path.
    #[cfg(not(feature = "dogecoin"))]
    cursor.set_position(HEADER_LEN as u64);
    #[cfg(feature = "dogecoin")]
    {
        let _ = bitcoin::dogecoin::Header::consensus_decode_from_finite_reader(&mut cursor)?;
    }

    // `from_finite_reader` skips the `Take<R>` wrap that
    // `consensus_decode` applies to every nested field for memory
    // safety: our cursor is already a bounded `Vec<u8>`, so the
    // wrapping is pure overhead and compounds across ~2000 tx fields.
    let tx_count = VarInt::consensus_decode_from_finite_reader(&mut cursor)?.0 as usize;
    let mut txdata = Vec::with_capacity(tx_count);
    let mut tx_metadata = Vec::with_capacity(tx_count);
    let mut tx_offsets = Vec::with_capacity(tx_count);
    for _ in 0..tx_count {
        let tx_start = cursor.position() as u32;
        tx_offsets.push(tx_start);
        let tx = Transaction::consensus_decode_from_finite_reader(&mut cursor)?;
        let tx_len = cursor.position() as u32 - tx_start;
        txdata.push(tx);
        tx_metadata.push(BlkMetadata::new(metadata.position() + tx_start, tx_len));
    }

    let raw_bytes = cursor.into_inner();
    #[cfg(not(feature = "litecoin"))]
    let primitive_block = bitcoin::Block { header, txdata };
    // Litecoin's `Block` carries the MWEB extension block; the indexer
    // ignores it (peg-in/peg-out are visible as normal txs), so default
    // it to `None`.
    #[cfg(feature = "litecoin")]
    let primitive_block = bitcoin::Block {
        header,
        txdata,
        mweb_block: None,
    };
    let mut block = Block::from((height, bitcoin_hash, primitive_block));
    block.set_raw_data(raw_bytes, tx_offsets);
    Ok(ReadBlock::from((block, metadata, tx_metadata)))
}

#[cfg(all(test, feature = "dogecoin"))]
mod tests {
    use super::*;
    use brk_types::BlkPosition;

    fn decode_hex(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }

    /// Runs a full block buffer through the same steps as the pipeline:
    /// decode the pure 80-byte header (as `peek_canonical` does), then parse
    /// the body. Asserts the tx count and block hash match the authoritative
    /// AuxPoW-aware `dogecoin::Block` decoder.
    fn assert_parses(hex: &str, height: u32, expect_auxpow: bool) {
        let bytes = decode_hex(hex);

        // Authoritative reference decode (consumes AuxPoW when present).
        let reference =
            bitcoin::dogecoin::Block::consensus_decode(&mut &bytes[..]).expect("reference decode");
        assert_eq!(
            reference.header.aux_pow.is_some(),
            expect_auxpow,
            "auxpow presence mismatch for height {height}"
        );

        // Pure header from the first 80 bytes, matching `peek_canonical`.
        let header = Header::consensus_decode_from_finite_reader(&mut &bytes[..HEADER_LEN])
            .expect("pure header decode");

        let metadata = BlkMetadata::new(BlkPosition::new(0, 0), bytes.len() as u32);
        let xor_state = XORIndex::at_offset(0);
        let xor_bytes = XORBytes::from([0u8; 8]);

        let read_block = parse_canonical_body(
            bytes,
            metadata,
            xor_state,
            xor_bytes,
            Height::new(height),
            header,
        )
        .expect("parse_canonical_body");

        assert_eq!(
            read_block.txdata.len(),
            reference.txdata.len(),
            "tx count mismatch for height {height}"
        );
        assert!(read_block.txdata.len() >= 1, "block must have a coinbase tx");
        assert_eq!(
            read_block.hash(),
            &BlockHash::from(reference.block_hash()),
            "block hash mismatch for height {height}"
        );
    }

    /// Legacy pre-AuxPoW block (version 0x01): the tx-count varint sits
    /// immediately after the 80-byte header, exercising the non-AuxPoW path.
    #[test]
    fn parses_legacy_pre_auxpow_block() {
        let hex = "01000000c76fe7f8ec09989d32b7907966fbd347134f80a7b71efce55fec502aa126ba3894b3065289ff8ba1ab4e8391771174d47cf2c974ebd24a1bdafd6c107d5a7a207d78bb52de8f001c00da8c3c0201000000010000000000000000000000000000000000000000000000000000000000000000ffffffff2602bc6d062f503253482f047178bb5208f8042975030000000d2f7374726174756d506f6f6c2f000000000100629b29c45500001976a91450e9fe87c705dcd4b7523b47e3314c2115f5d5df88ac0000000001000000015f48fabf4425324df2b5e58f4e9c771297f76f5fa37db7556f6fc1d22742da1f010000006a473044022062d29d2d26f7d826e7b72257486e294d284832743c7803a2901eb07e326b25a002207efc391b0f4e724c9d518075c0e056cc425540f845b0fd419ba8a9d49d69288301210297a2568525760a98454d84f5e5adba9fd0a41726a6fb774ddc407279e41e2061ffffffff0240bab598200000001976a91401348a2b83aeb6b1ba2a174a1a40b7c75fbeb12088ac0040be40250000001976a914025407d928ef333979d064ae233353d80e29d58c88ac00000000";
        assert_parses(hex, 100_000, false);
    }

    /// Merge-mined block (version 0x00620102, AuxPoW bit set): a variable-length
    /// AuxPoW payload sits between the 80-byte header and the tx-count varint.
    /// This is the case that panics under the old fixed-80-byte assumption.
    #[test]
    fn parses_merge_mined_auxpow_block() {
        let hex = "020162001e21ad14bc1ef20cf2d58e2b755ae4a7bfb75c906c74ef3dbb97cc57dcd77581b14423c43517df2b4f3277731daba29d0d865b515a6f19f5fb61d5799b28f2c9b48713540fa8071b0000000001000000010000000000000000000000000000000000000000000000000000000000000000ffffffff4f032ac209fabe6d6dd3ea48350b102b90acf9eac6629072d5f697c02faf360b26d365e7b2bfb980700100000000000000062f503253482f04ce871354080811312915000000092f7374726174756d2f000000000100f2052a010000001976a914f332ec6f1729495e7edcd8ce9d887742567fe60988ac0000000006d2bbd93141ea6d2c8434caeb01828a2a522275b66d2b21fe4ed8230cfe65a101ad2f07c348abdc05f57e2e7d8763488aad71df9f557aec46ae0207ae2bb74a1500000000000000000002000000f288b555ed9b44c814afbbbac135d95e0984a5cc7cb554fccbd2ca27c5e423cebf3021ed058ac83eaa0f64b0d405fc99216209b7e56deeeefceb3629210d1cabcb8713545a50021bc40227b50301000000010000000000000000000000000000000000000000000000000000000000000000ffffffff0d0300b1050101062f503253482fffffffff010085fd36af050000232102c9cbaeb767cc8c884204601f322c6977890cdd3d274f8b1a704ae00382102191ac0000000001000000011fcccc77028fc5fb96d60ee5f258da271ba9b4fdec12594a5dd2efa1fb5bd14b010000006b483045022100f4a17664176706f74877433dbfb8e68a5c1e45730da9248a8da3bab833cea1ca022018fee77621c20f196b82c3c8fd663959b0c8ea985b0407bdbac1414a5a5a21bf0121033fcc1cb9c1b7b11758eb2cd3a25b4ff917a5e248f5d6fcc74160dd6a450acf8bffffffff020759dd6e000000001976a914a553c686ac1aa534ffcb3b694c463944175a6f3c88ac8548f276890700001976a914a1ea13863020f36897b671ad328d98e9364f12b488ac000000000100000001215c45fc31d3beae4a5c76efbb0925d0b4bace72f62248aa3777927eecb42152000000006b483045022100bd3f18da6acd8180ed99c7c7fc6feab18653008cc58be5c32a470193aea330860220719c64121805606240c48dab2e8e3f3d82501b78b78597043ca134abf4c85443012103e6435d9ad2a3f3ff2d2c58aef41075df4eb5417c313d3ea4d5bb87ab46241940ffffffff0140ab8aac140000001976a91481db1aa49ebc6a71cad96949eb28e22af85eb0bd88ac00000000";
        assert_parses(hex, 371_337, true);
    }
}
