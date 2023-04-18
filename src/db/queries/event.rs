use super::*;

use sqlx::{self};

impl Queries {
    #[allow(clippy::too_many_arguments)]
    pub async fn list_events(
        &self,
        nft: Option<&String>,
        collections: &[String],
        owner: Option<&String>,
        event_type: &[NftEventType],
        category: &[NftEventCategory],
        offset: usize,
        limit: usize,
        with_count: bool,
        verified: Option<bool>,
    ) -> sqlx::Result<NftEventsRecord> {
        let event_types_slice = &event_type
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()[..];
        let categories_slice = &category
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()[..];

        sqlx::query_file_as!(
            NftEventsRecord,
            "src/db/sql/list_activities.sql",
            categories_slice,
            event_types_slice,
            owner,
            nft,
            collections,
            limit as i64,
            offset as i64,
            with_count,
            verified
        )
        .fetch_one(self.db.as_ref())
        .await
    }
}
