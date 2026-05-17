use crate::{data_definition::table::Identifier, BuildSql};
use sqlx::{Postgres, QueryBuilder};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};
use uuid::Uuid;

#[derive(Clone)]
// TODO: This is duplicated iwth DB types somewhere else
pub enum FilterComparisonParam {
    TableColumn(Identifier),
    String(String),
    Uuid(Uuid),
    Integer(i64),
    Float(f64),
    Bool(bool),
    Timestamp(chrono::NaiveDateTime),
    Null,
}

impl FilterComparisonParam {
    pub fn build_sql(
        &self,
        builder: &mut QueryBuilder<Postgres>,
    ) {
        match self {
            FilterComparisonParam::TableColumn(col) => builder.push(col),
            FilterComparisonParam::String(val) => builder.push_bind(val.to_string()),
            FilterComparisonParam::Uuid(val) => builder.push_bind(*val),
            FilterComparisonParam::Integer(val) => builder.push_bind(*val),
            FilterComparisonParam::Float(val) => builder.push_bind(*val),
            FilterComparisonParam::Bool(val) => builder.push_bind(*val),
            FilterComparisonParam::Timestamp(val) => builder.push_bind(*val),
            FilterComparisonParam::Null => builder.push("null"),
        };
    }
}

#[derive(Clone)]
// TODO: Make Filters associated with their tables
// There's a lot more to do with the filters here - nailing this down is gonna be super powerful
// Will work with PG stuff, and also let me piecemeal replace the functionality...
pub enum Filter {
    And(Vec<Filter>),
    Or(Vec<Filter>),
    Equal(FilterComparisonParam, FilterComparisonParam), // All types
    NotEqual(FilterComparisonParam, FilterComparisonParam), // All types
    Like(FilterComparisonParam, FilterComparisonParam),  // Strings only
    LessThan(FilterComparisonParam, FilterComparisonParam), // Non-String types (dates, numbers)
    LessThanOrEqual(FilterComparisonParam, FilterComparisonParam), // Non-String types (dates, numbers)
    GreaterThan(FilterComparisonParam, FilterComparisonParam),     // Non-String types
    GreaterThanOrEqual(FilterComparisonParam, FilterComparisonParam), // Non-String types
    In(FilterComparisonParam, Vec<FilterComparisonParam>),         // All types
}

impl Filter {
    fn get_operator(&self) -> &str {
        match self {
            Filter::And(_) => "AND",
            Filter::Or(_) => "OR",
            Filter::Equal(_, _) => "=",
            Filter::NotEqual(_, _) => "!=",
            Filter::Like(_, _) => "LIKE",
            Filter::LessThan(_, _) => "<",
            Filter::LessThanOrEqual(_, _) => "<=",
            Filter::GreaterThan(_, _) => ">",
            Filter::GreaterThanOrEqual(_, _) => ">=",
            Filter::In(_, _) => "IN",
        }
    }
}

// trait Likeable {
//     fn like<T: Into<String>>(
//         &self,
//         _t: T,
//     ) -> Filter {
//         Filter::Like(
//             FilterComparisonParam::TableColumn(todo!()),
//             FilterComparisonParam::String(_t.into()),
//         )
//     }
// }

impl BitAndAssign for Filter {
    fn bitand_assign(
        &mut self,
        rhs: Self,
    ) {
        *self = self.clone() & rhs;
    }
}

impl BitOrAssign for Filter {
    fn bitor_assign(
        &mut self,
        rhs: Self,
    ) {
        *self = self.clone() | rhs;
    }
}

// TODO: Refactor BitOr and BitAnd to actually merge them together, instead of wrapping them in a higher level construct - applies only if either one is already an Or/And
impl BitOr for Filter {
    type Output = Filter;
    fn bitor(
        self,
        rhs: Self,
    ) -> Self::Output {
        match self {
            Filter::Or(mut existing_ors) => {
                existing_ors.push(rhs);
                Filter::Or(existing_ors)
            },
            _ => Filter::Or(vec![self, rhs]),
        }
    }
}

impl BitAnd for Filter {
    type Output = Filter;
    fn bitand(
        self,
        rhs: Self,
    ) -> Self::Output {
        match self {
            Filter::And(mut existing_ors) => {
                existing_ors.push(rhs);
                Filter::And(existing_ors)
            },
            _ => Filter::And(vec![self, rhs]),
        }
    }
}

impl BuildSql for Filter {
    fn build_sql(
        &self,
        builder: &mut QueryBuilder<Postgres>,
    ) {
        match self {
            Filter::And(children) => {
                let mut iter = children.iter().peekable();
                builder.push("(");
                while let Some(child) = iter.next() {
                    child.build_sql(builder);
                    if iter.peek().is_some() {
                        builder.push(" AND ");
                    }
                }
                builder.push(")");
            },
            Filter::Or(children) => {
                let mut iter = children.iter().peekable();
                builder.push("(");
                while let Some(child) = iter.next() {
                    child.build_sql(builder);
                    if iter.peek().is_some() {
                        builder.push(" OR ");
                    }
                }
                builder.push(")");
            },
            Filter::Equal(l, r)
            | Filter::NotEqual(l, r)
            | Filter::Like(l, r)
            | Filter::LessThan(l, r)
            | Filter::LessThanOrEqual(l, r)
            | Filter::GreaterThan(l, r)
            | Filter::GreaterThanOrEqual(l, r) => {
                l.build_sql(builder);
                builder.push(" ");
                builder.push(self.get_operator());
                builder.push(" ");
                r.build_sql(builder);
            },
            Filter::In(_val, _list) => {
                todo!()
            },
        }
    }
}
