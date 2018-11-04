table! {
    vote_opportunities (id) {
        id -> Int4,
        title -> Text,
        location_name -> Nullable<Text>,
        lat -> Float8,
        lng -> Float8,
        description -> Text,
        date -> Timestamp,
        tags -> Array<Text>,
    }
}
