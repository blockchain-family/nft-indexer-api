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
        _verified: Option<bool>,
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
        )
            .fetch_one(self.db.as_ref())
            .await
    }

    pub async fn list_events_count(
        &self,
        nft: Option<&String>,
        collection: Option<&String>,
        _owner: Option<&String>,
        typ: &[EventType],
    ) -> sqlx::Result<i64> {
        let typ_str: Vec<String> = typ.iter().map(|x| x.to_string()).collect();
        sqlx::query!(
            "
            SELECT count(*)
            FROM nft_events e
            WHERE
                ($1::varchar is null OR e.nft = $1)
                AND ($2::varchar is null OR e.collection = $2)
                AND (array_length($3::varchar[], 1) is null OR e.event_type::varchar = ANY($3))
            ",
            nft,
            collection,
            &typ_str
        )
            .fetch_one(self.db.as_ref())
            .await
            .map(|r| r.count.unwrap_or_default())
    }

}
