pub mod delete;
pub mod insert;
pub mod update;
// pub mod upsert;

#[allow(unused)]
pub(crate) trait SqlStatement {
    fn to_sql() -> String;
}
