use crate::{error::Error, utils::*, Result};
use iso8601_timestamp::{Duration, Timestamp};
use oberon::{Proof, PublicKey};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiProof {
    #[serde(
        serialize_with = "serialize_uuid",
        deserialize_with = "deserialize_uuid"
    )]
    pub id: Uuid,
    pub proof: Proof,
    pub timestamp: i64,
}

impl ApiProof {
    /// Verify an API proof
    pub fn verify(&self, pk: PublicKey) -> Result<()> {
        let proof_timestamp =
            Timestamp::UNIX_EPOCH.saturating_add(Duration::milliseconds(self.timestamp));
        let timestamp = Timestamp::now_utc();

        if timestamp
            .duration_since(proof_timestamp)
            .whole_milliseconds()
            > 5000
        {
            return Err(Error::ExpiredOberonProof);
        }

        let result = self
            .proof
            .open(pk, self.id.as_bytes(), self.timestamp.to_be_bytes());

        if result.unwrap_u8() == 1u8 {
            Ok(())
        } else {
            Err(Error::InvalidOberonProof)
        }
    }
}
