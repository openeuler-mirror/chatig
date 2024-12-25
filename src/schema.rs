// @generated automatically by Diesel CLI.

diesel::table! {
    file_object (id) {
        id -> Int4,
        object -> Text,
        bytes -> Int4,
        created_at -> Int8,
        filename -> Text,
        purpose -> Text,
    }
}

diesel::table! {
    invitation_code (id) {
        id -> Int4,
        users -> Text,
        origination -> Nullable<Text>,
        telephone -> Nullable<Text>,
        email -> Nullable<Text>,
        created_at -> Int8,
        code -> Text,
    }
}

diesel::table! {
    project_object (id) {
        id -> Text,
        object -> Text,
        name -> Text,
        created_at -> Int8,
        archived_at -> Nullable<Int8>,
        status -> Text,
    }
}

diesel::table! {
    user_object (id) {
        id -> Text,
        object -> Text,
        name -> Text,
        email -> Text,
        role -> Text,
        added_at -> Int8,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    file_object,
    invitation_code,
    project_object,
    user_object,
);
