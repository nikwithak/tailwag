use std::fmt::Debug;
use std::str::FromStr;

use crate::application::http::route::HttpBody;
use crate::application::http::route::Request;
use crate::application::http::route::RequestContext;
use crate::auth::gateway::AppUser;
use crate::option_utils::OrError;
use crate::HttpResult;
use serde::Deserialize;
use serde::Serialize;
use tailwag_orm::data_definition::table::Identifier;
use tailwag_orm::data_manager::rest_api::Id;
use tailwag_orm::data_manager::traits::DataProvider;
use tailwag_orm::data_manager::GetTableDefinition;
use tailwag_orm::data_manager::PostgresDataProvider;
use tailwag_orm::queries::filterable_types::Filterable;
use tailwag_orm::queries::Deleteable;
use tailwag_orm::queries::Filter;
use tailwag_orm::queries::FilterComparisonParam;
use tailwag_orm::queries::Insertable;
use tailwag_orm::queries::Updateable;
use uuid::Uuid;

mod tailwag {
    pub use crate as web;
    pub use tailwag_forms as forms;
    pub use tailwag_macros as macros;
    pub use tailwag_orm as orm;
}

#[derive(
    Clone,
    Debug,
    Default,
    serde::Deserialize,
    serde::Serialize,
    tailwag_orm_macros::GetTableDefinition,
    tailwag_orm_macros::Insertable,
    tailwag_orm_macros::Updateable,
    tailwag_orm_macros::Deleteable,
    tailwag_orm_macros::Filterable,
    tailwag::macros::BuildRoutes,
    tailwag_orm_macros::Id,
    tailwag::macros::Display,
    tailwag::forms::macros::GetForm,
)]
#[no_default_routes]
pub struct Comment {
    id: uuid::Uuid,
    pub(crate) message: String,
    #[no_filter]
    #[create_ignore]
    #[ref_only]
    pub(crate) created_by: AppUser,
    #[create_ignore]
    pub(crate) track_id: uuid::Uuid,
    #[create_ignore]
    pub(crate) create_date: chrono::NaiveDateTime,
    #[create_ignore]
    pub(crate) last_modified_date: Option<chrono::NaiveDateTime>,
    #[create_ignore]
    pub(crate) parent_comment_id: Option<uuid::Uuid>,
}

#[deprecated = "Currently broken - do not use. Copy this function manually instead."]
pub async fn post_comment<T: Commentable>(
    req: Request,
    req_ctx: RequestContext,
) -> HttpResult<Comment>
where
    T: Updateable
        + 'static
        + Deleteable
        + Send
        + Serialize
        + for<'a> Deserialize<'a>
        + Clone
        + Unpin
        + Id
        + Insertable
        + Filterable
        + Sync
        + Debug
        + GetTableDefinition
        + Default,
{
    let t_items: PostgresDataProvider<T> = (&req_ctx).into();

    let user: AppUser = req_ctx.get_request_data::<AppUser>().or_401()?.clone();

    let id = req.path_params.first().and_then(|id| Uuid::from_str(id).ok()).or_404()?;
    let comment: CommentCreateRequest = match req.body {
        HttpBody::Json(body) => serde_json::from_str(&body)?,
        _ => crate::HttpError::unsupported_media_type()?,
    };
    let mut comment: Comment = comment.into();
    comment.create_date = chrono::Utc::now().naive_utc();
    comment.created_by = user;
    comment.track_id = id;

    let table_name = T::get_table_definition().table_name;
    let filter = Filter::Equal(
        FilterComparisonParam::TableColumn(Identifier::new_unchecked(format!(
            "{}.id",
            &table_name
        ))),
        FilterComparisonParam::Uuid(id),
    );
    let mut item = t_items.get(|_item| filter.clone()).await?.or_404()?;
    item.add_comment(comment.clone());
    log::error!("Adding new comment:{:?} to: {:?}", &comment, &item);
    t_items.update(&item).await?;
    Ok(comment)
}

// TODO: Derive macro for this
pub trait Commentable {
    fn add_comment(
        &mut self,
        comment: Comment,
    );
}
