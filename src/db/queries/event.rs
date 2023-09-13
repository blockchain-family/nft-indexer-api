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
        _category: &[NftEventCategory],
        offset: usize,
        limit: usize,
        with_count: bool,
        _verified: Option<bool>,
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
            r#"
            select count(*)
            from nft_events e
            where ($1::varchar is null or e.nft = $1)
              and ($2::varchar is null or e.collection = $2)
              and (array_length($3::varchar[], 1) is null or e.event_type::varchar = any ($3))
            "#,
            nft,
            collection,
            &typ_str
        )
        .fetch_one(self.db.as_ref())
        .await
        .map(|r| r.count.unwrap_or_default())
    }
}
