// @generated automatically by Diesel CLI.

diesel::table! {
    files (id) {
        id -> Integer,
        name -> Text,
        status -> Integer,
        frozen -> Nullable<Integer>,
        sha2 -> Text,
        last_update -> Nullable<Timestamp>,
    }
}
