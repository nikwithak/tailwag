#[allow(unused)]
pub enum ApplicationComponent {
    Service,
    Route,
    DataSource,
}

#[allow(unused)]
pub struct PostgresDataSourceConfig {
    conn_string: String,
    max_connections: u32,
}
