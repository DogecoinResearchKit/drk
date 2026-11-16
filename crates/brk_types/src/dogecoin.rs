//! Dogecoin's merge-mined blocks (from 371,337) put an AuxPoW proof between the 80-byte header
//! and the transaction count.

use ::bitcoin::{
    block::Header,
    consensus::{Decodable, encode},
    dogecoin::auxpow::AuxPow,
    io::Read,
};

/// Reads past the AuxPoW that follows `header` when it has one, returning the bytes consumed.
pub fn skip_auxpow<R: Read + ?Sized>(
    header: &Header,
    reader: &mut R,
) -> Result<usize, encode::Error> {
    if !header.has_auxpow_bit() {
        return Ok(0);
    }
    let aux_pow = AuxPow::consensus_decode_from_finite_reader(reader)?;
    Ok(::bitcoin::consensus::serialize(&aux_pow).len())
}
