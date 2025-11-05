use sqlx::{self};

use super::*;

impl Queries {
    #[allow(clippy::too_many_arguments)]
    pub async fn list_events(
        &self,
        nft: Option<&String>,
        collections: &[String],
        owner: Option<&String>,
        event_type: &[NftEventType],
        offset: usize,
        limit: usize,
        with_count: bool,
        verified: Option<bool>,
    ) -> sqlx::Result<NftEventsRecord> {
        let event_types_slice = &event_type
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()[..];

        sqlx::query_file_as!(
            NftEventsRecord,
            "src/db/sql/activities.sql",
            event_types_slice as _,
            owner as _,
            nft as _,
            collections as _,
            limit as i64,
            offset as i64,
            with_count,
            verified
        )
        .fetch_one(self.db.as_ref())
        .await
    }
}
