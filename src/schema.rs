table! {
    searches (id) {
        id -> Uuid,
        url -> Nullable<Text>,
        result_url -> Nullable<Text>,
        email -> Nullable<Varchar>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}
