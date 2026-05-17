use std::str::FromStr;

use crate::application::http::multipart::FromMultipartRequest;
use crate::application::http::route::IntoResponse;
use crate::application::http::route::ServerData;
use crate::extras::image_upload::GetFileDetails;
use crate::extras::image_upload::MimeType;
use crate::option_utils::OrError;
use crate::HttpResult;
use crate::{
    application::http::route::{FromRequest, PathString, Response},
    Error,
};
use serde::Deserialize;
use serde::Serialize;
use tailwag_orm::data_definition::table::Identifier;
use tailwag_orm::data_manager::local_storage_provider::LocalStorageFileProvider;
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

#[derive(Clone)]
pub struct File<T: FromMultipartRequest> {
    #[allow(unused)]
    pub mime_type: MimeType,
    pub filename: String,
    pub bytes: Vec<u8>,
    pub associated_data: T,
}

///
pub async fn load_file<T>(
    id: PathString,
    t_provider: PostgresDataProvider<T>,
    ServerData(file_provider): ServerData<LocalStorageFileProvider>,
) -> HttpResult<Response>
where
    T: Insertable,
    T: FileKey,
    <T as Insertable>::CreateRequest: FromMultipartRequest,
    // This next bit is a chain of requirements for PostgresDataProvider.
    // TODO [TECH DEBT]: Eventually I'll clean this up so that DataProvider is more flexible.
    T: Updateable
        + Deleteable
        + Send
        + Serialize
        + for<'a> Deserialize<'a>
        + Clone
        + Unpin
        + Id
        + GetTableDefinition
        + Filterable
        + Default,
{
    // A bit hacky...
    let col_name = format!("{}.id", &T::get_table_definition().table_name);
    let filter = Filter::Equal(
        FilterComparisonParam::TableColumn(Identifier::new_unchecked(col_name)),
        FilterComparisonParam::Uuid(
            Uuid::from_str(&*id)
                .map_err(|_| crate::Error::BadRequest("Invalid UUID provided".into()))?,
        ),
    );

    // TODO: Fix this filter, this is 404ing
    let t = t_provider.get(|_| filter.clone()).await?.or_404()?;

    let file_key = t.get_file_key().or_404()?;

    let bytes = file_provider.read_file(file_key)?;

    Ok(Response::ok().with_body(bytes).with_header(
        "content-type",
        MimeType::try_from_filename(file_key)
            .map(|mt| mt.to_string())
            .unwrap_or("application/octet-stream".to_string()),
    ))
}

/// A custom implementation of FromRequest allows us to parse the Multipart bits for Image.
/// This is a result of the choice to base a lot of things off of Serde, and generically implement FromRequest for T: Serialize.
impl<T: FromMultipartRequest + Sized> FromRequest for File<T> {
    fn from(req: crate::application::http::route::Request) -> Result<Self, Error> {
        let result = match req.body {
            crate::application::http::route::HttpBody::Multipart(mut parts) => {
                let file = parts.remove("file").ok_or(Error::BadRequest("Missing file".into()))?;
                let mime_type = file
                    .get_mime_type()
                    .ok_or(Error::BadRequest("File is not a supported type.".into()))?;
                let filename = file
                    .get_filename()
                    .ok_or(Error::BadRequest("no filename for file".into()))?
                    .to_string();

                let metadata = T::from_multipart_request(&parts);

                File::<T> {
                    bytes: file.content,
                    mime_type,
                    filename,
                    associated_data: metadata?,
                }
            },
            _ => Err(crate::Error::BadRequest(
                "This endpoint requires multipart/form_data.".to_owned(),
            ))?,
        };
        Ok(result)
    }
}

pub async fn save_file<T>(
    file: File<<T as Insertable>::CreateRequest>,
    t_provider: PostgresDataProvider<T>,
    ServerData(file_provider): ServerData<LocalStorageFileProvider>,
) -> HttpResult<Response>
where
    T: Insertable,
    T: FileKey,
    <T as Insertable>::CreateRequest: FromMultipartRequest,
    // This next bit is a chain of requirements for PostgresDataProvider.
    // TODO [TECH DEBT]: Eventually I'll clean this up so that DataProvider is more flexible.
    T: Updateable
        + Deleteable
        + Send
        + Serialize
        + for<'a> Deserialize<'a>
        + Clone
        + Unpin
        + Id
        + Filterable
        + Default,
{
    let inferred_mime_type =
        MimeType::try_from_filename(file.filename.as_str()).unwrap_or_default();
    if file.mime_type != inferred_mime_type {
        crate::Error::bad_request("File type doesn't match provided mime_type")?;
    }
    file_provider.save_file(&file.filename, file.bytes)?;
    let mut t = t_provider.create(file.associated_data).await?;
    t.set_file_key(&file.filename);
    t_provider.update(&t).await?;

    Ok(t.into_response())
}

pub trait FileKey {
    fn set_file_key(
        &mut self,
        key: &str,
    );
    fn get_file_key(&self) -> Option<&str>;
}
