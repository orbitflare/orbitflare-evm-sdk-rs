use alloy_primitives::{Address, B256};
use orbitflare_polygon_proto::common::{H128, H160, H256};

fn h128_bytes(h: &H128) -> [u8; 16] {
    let mut out = [0u8; 16];
    out[..8].copy_from_slice(&h.hi.to_be_bytes());
    out[8..].copy_from_slice(&h.lo.to_be_bytes());
    out
}

fn h128_from(bytes: &[u8]) -> H128 {
    H128 {
        hi: u64::from_be_bytes(bytes[..8].try_into().unwrap()),
        lo: u64::from_be_bytes(bytes[8..16].try_into().unwrap()),
    }
}

pub trait ToAlloy {
    type Out;
    fn to_alloy(&self) -> Self::Out;
}

impl ToAlloy for H256 {
    type Out = B256;

    fn to_alloy(&self) -> B256 {
        let mut out = [0u8; 32];
        if let Some(hi) = &self.hi {
            out[..16].copy_from_slice(&h128_bytes(hi));
        }
        if let Some(lo) = &self.lo {
            out[16..].copy_from_slice(&h128_bytes(lo));
        }
        B256::from(out)
    }
}

impl ToAlloy for H160 {
    type Out = Address;

    fn to_alloy(&self) -> Address {
        let mut out = [0u8; 20];
        if let Some(hi) = &self.hi {
            out[..16].copy_from_slice(&h128_bytes(hi));
        }
        out[16..].copy_from_slice(&self.lo.to_be_bytes());
        Address::from(out)
    }
}

pub trait ToProto {
    type Out;
    fn to_proto(&self) -> Self::Out;
}

impl ToProto for B256 {
    type Out = H256;

    fn to_proto(&self) -> H256 {
        let bytes = self.as_slice();
        H256 {
            hi: Some(h128_from(&bytes[..16])),
            lo: Some(h128_from(&bytes[16..])),
        }
    }
}

impl ToProto for Address {
    type Out = H160;

    fn to_proto(&self) -> H160 {
        let bytes = self.as_slice();
        H160 {
            hi: Some(h128_from(&bytes[..16])),
            lo: u32::from_be_bytes(bytes[16..20].try_into().unwrap()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{address, b256};

    #[test]
    fn b256_round_trip() {
        let original = b256!("0x1a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c4d5e6f708192a3b4c5d6e7f809");
        assert_eq!(original.to_proto().to_alloy(), original);
    }

    #[test]
    fn address_round_trip() {
        let original = address!("0x1a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c4d");
        assert_eq!(original.to_proto().to_alloy(), original);
    }

    #[test]
    fn zero_values() {
        assert_eq!(B256::ZERO.to_proto().to_alloy(), B256::ZERO);
        assert_eq!(Address::ZERO.to_proto().to_alloy(), Address::ZERO);
    }
}
