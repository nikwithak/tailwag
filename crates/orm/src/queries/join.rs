// TODO: Not in use yet.

use crate::data_definition::table::Identifier;

#[allow(unused)]
pub struct Join {
    pub(crate) left_table: Identifier,
    pub(crate) left_table_alias: Identifier,
    pub(crate) right_table: Identifier,
    pub(crate) right_table_alias: Identifier,
    pub(crate) left_table_join_column: Identifier,
    pub(crate) right_table_join_column: Identifier,
    pub(crate) join_side: JoinSide,
    pub(crate) join_type: JoinType,
}

pub enum JoinSide {
    Left,
    Right,
}

pub enum JoinType {
    Inner,
    Outer,
}
