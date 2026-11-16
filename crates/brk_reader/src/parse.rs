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

    const LEGACY_100_000: &str = "01000000c76fe7f8ec09989d32b7907966fbd347134f80a7b71efce55fec502aa126ba3894b3065289ff8ba1ab4e8391771174d47cf2c974ebd24a1bdafd6c107d5a7a207d78bb52de8f001c00da8c3c0201000000010000000000000000000000000000000000000000000000000000000000000000ffffffff2602bc6d062f503253482f047178bb5208f8042975030000000d2f7374726174756d506f6f6c2f000000000100629b29c45500001976a91450e9fe87c705dcd4b7523b47e3314c2115f5d5df88ac0000000001000000015f48fabf4425324df2b5e58f4e9c771297f76f5fa37db7556f6fc1d22742da1f010000006a473044022062d29d2d26f7d826e7b72257486e294d284832743c7803a2901eb07e326b25a002207efc391b0f4e724c9d518075c0e056cc425540f845b0fd419ba8a9d49d69288301210297a2568525760a98454d84f5e5adba9fd0a41726a6fb774ddc407279e41e2061ffffffff0240bab598200000001976a91401348a2b83aeb6b1ba2a174a1a40b7c75fbeb12088ac0040be40250000001976a914025407d928ef333979d064ae233353d80e29d58c88ac00000000";

    const AUXPOW_371_337: &str = "020162001e21ad14bc1ef20cf2d58e2b755ae4a7bfb75c906c74ef3dbb97cc57dcd77581b14423c43517df2b4f3277731daba29d0d865b515a6f19f5fb61d5799b28f2c9b48713540fa8071b0000000001000000010000000000000000000000000000000000000000000000000000000000000000ffffffff4f032ac209fabe6d6dd3ea48350b102b90acf9eac6629072d5f697c02faf360b26d365e7b2bfb980700100000000000000062f503253482f04ce871354080811312915000000092f7374726174756d2f000000000100f2052a010000001976a914f332ec6f1729495e7edcd8ce9d887742567fe60988ac0000000006d2bbd93141ea6d2c8434caeb01828a2a522275b66d2b21fe4ed8230cfe65a101ad2f07c348abdc05f57e2e7d8763488aad71df9f557aec46ae0207ae2bb74a1500000000000000000002000000f288b555ed9b44c814afbbbac135d95e0984a5cc7cb554fccbd2ca27c5e423cebf3021ed058ac83eaa0f64b0d405fc99216209b7e56deeeefceb3629210d1cabcb8713545a50021bc40227b50301000000010000000000000000000000000000000000000000000000000000000000000000ffffffff0d0300b1050101062f503253482fffffffff010085fd36af050000232102c9cbaeb767cc8c884204601f322c6977890cdd3d274f8b1a704ae00382102191ac0000000001000000011fcccc77028fc5fb96d60ee5f258da271ba9b4fdec12594a5dd2efa1fb5bd14b010000006b483045022100f4a17664176706f74877433dbfb8e68a5c1e45730da9248a8da3bab833cea1ca022018fee77621c20f196b82c3c8fd663959b0c8ea985b0407bdbac1414a5a5a21bf0121033fcc1cb9c1b7b11758eb2cd3a25b4ff917a5e248f5d6fcc74160dd6a450acf8bffffffff020759dd6e000000001976a914a553c686ac1aa534ffcb3b694c463944175a6f3c88ac8548f276890700001976a914a1ea13863020f36897b671ad328d98e9364f12b488ac000000000100000001215c45fc31d3beae4a5c76efbb0925d0b4bace72f62248aa3777927eecb42152000000006b483045022100bd3f18da6acd8180ed99c7c7fc6feab18653008cc58be5c32a470193aea330860220719c64121805606240c48dab2e8e3f3d82501b78b78597043ca134abf4c85443012103e6435d9ad2a3f3ff2d2c58aef41075df4eb5417c313d3ea4d5bb87ab46241940ffffffff0140ab8aac140000001976a91481db1aa49ebc6a71cad96949eb28e22af85eb0bd88ac00000000";
}
