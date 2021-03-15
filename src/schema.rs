table! {
    commands (id) {
        id -> Unsigned<Bigint>,
        serial -> Varchar,
        completed_at -> Nullable<Datetime>,
    }
}

table! {
    external_monitors (id) {
        id -> Unsigned<Bigint>,
        created_at -> Datetime,
        serial -> Varchar,
        cpu_usage -> Unsigned<Tinyint>,
        memory_usage -> Unsigned<Tinyint>,
        disk_usage -> Unsigned<Tinyint>,
        status -> Nullable<Text>,
    }
}

table! {
    monitors (id) {
        id -> Unsigned<Bigint>,
        #[sql_name = "type"]
        type_ -> Varchar,
        frequency_min -> Unsigned<Smallint>,
        endpoint -> Varchar,
        max_latency_ms -> Nullable<Unsigned<Integer>>,
        expected_ip -> Nullable<Varchar>,
        minimum_expiration_time_d -> Nullable<Unsigned<Integer>>,
    }
}

table! {
    status (id) {
        id -> Unsigned<Bigint>,
        created_at -> Datetime,
        monitor_id -> Unsigned<Bigint>,
        succeed -> Bool,
        response_time_ms -> Unsigned<Integer>,
        result -> Varchar,
    }
}

allow_tables_to_appear_in_same_query!(
    commands,
    external_monitors,
    monitors,
    status,
);
