pub mod aggregator;
pub mod api_proof;
pub mod config;
pub mod error;
pub mod platform;
pub mod source;
pub mod utils;

pub(crate) mod orderbook {
    use super::{api_proof::ApiProof, error::Error};
    use oberon::Proof;
    use std::fmt;
    include!("orderbook.rs");

    impl TryFrom<&OberonProof> for ApiProof {
        type Error = Error;

        fn try_from(value: &OberonProof) -> std::result::Result<Self, Self::Error> {
            if value.proof.len() != Proof::BYTES {
                return Err(Error::InvalidOberonProof);
            }
            let mut proof_bytes = [0u8; Proof::BYTES];
            proof_bytes.copy_from_slice(&value.proof[..]);
            let opt_proof = Proof::from_bytes(&proof_bytes);
            let proof = if opt_proof.is_some().unwrap_u8() == 1u8 {
                opt_proof.unwrap()
            } else {
                return Err(Error::InvalidOberonProof);
            };
            Ok(Self {
                id: value.id.parse()?,
                proof,
                timestamp: value.timestamp as i64,
            })
        }
    }

    impl fmt::Display for Summary {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let _ = writeln!(f, "Summary {{");
            let _ = writeln!(f, "\tspread: {}", self.spread);
            let _ = writeln!(f, "\tasks: {:?}", self.asks);
            let _ = writeln!(f, "\tbids: {:?}", self.bids);
            writeln!(f, "}}")
        }
    }
}

pub type Result<T> = error::Result<T>;
