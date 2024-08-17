use serde::{Deserialize, Serialize};
use stripe::StripeError;
use tailwag_orm::{
    data_manager::{traits::DataProvider, PostgresDataProvider},
    queries::{
        filterable_types::{FilterEq, Filterable, FilterableType},
        Filter,
    },
};
use tailwag_web_service::application::http::route::RoutePolicy;
use tailwag_web_service::{
    application::http::route::{FromRequest, IntoResponse, Request, Response, ServerData},
    extras::image_upload::{Image, ImageMetadata},
};
use uuid::Uuid;

// Needed to simulate  the consolidation library that doesn't actually exist in this scope.
// TODO: Fix this bloody thing, it's annoying.
mod tailwag {
    pub use tailwag_forms as forms;
    pub use tailwag_macros as macros;
    pub use tailwag_orm as orm;
    pub use tailwag_web_service as web;
}

#[derive(
    Clone,
    Debug,
    Default,
    serde::Deserialize,
    serde::Serialize,
    sqlx::FromRow,
    tailwag::macros::GetTableDefinition,
    tailwag::macros::Insertable,
    tailwag::macros::Updateable,
    tailwag::macros::Deleteable,
    tailwag::macros::Filterable,
    tailwag::macros::BuildRoutes,
    tailwag::macros::Id,
    tailwag::macros::Display,
    tailwag::forms::macros::GetForm,
)]
#[create_type(CreateProductRequest)]
#[post(create_product)]
#[actions(("/{id}/image", save_image, RoutePolicy::RequireAuthentication))]
#[get_policy(RoutePolicy::Public)]
#[list_policy(RoutePolicy::Public)]
// #[post_policy(RoutePolicy::RequireAuthentication)]
// #[delete_policy(RoutePolicy::RequireAuthentication)]
// #[patch_policy(RoutePolicy::RequireAuthentication)]
// #[delete_policy(RoutePolicy::RequireAuthentication)]
#[post_policy(RoutePolicy::Public)]
#[delete_policy(RoutePolicy::Public)]
#[patch_policy(RoutePolicy::Public)]
#[delete_policy(RoutePolicy::Public)]
// Can only upload an image AFTER the object has been created, due to current limitations with binary file uploads.
pub struct Product {
    #[no_form]
    id: Uuid,
    name: String,
    description: String,
    #[no_form]
    #[no_filter]
    stripe_price_id: Option<String>,
    #[no_form]
    #[no_filter]
    stripe_product_id: Option<String>,
    #[no_filter]
    image_metadata: Option<ImageMetadata>,
    price_usd_cents: i64,
}

impl From<&Product> for stripe::CreateCheckoutSessionLineItems {
    fn from(val: &Product) -> Self {
        stripe::CreateCheckoutSessionLineItems {
            adjustable_quantity: None,
            dynamic_tax_rates: None,
            price: val.stripe_price_id.clone(),
            price_data: None,
            quantity: Some(1), // TODO: Actually get the quantity.
            tax_rates: None,
        }
    }
}

/// Here is an example of overriding the CreateRequest type.
/// In this example, we want to create the product with the Stripe API
/// as a part of the create process, which we can't do without a custom type.
///
/// To accomplish this custom create implementation (Affecting the `POST` operation),
/// we do three things:
///
/// 1. Define the type, making sure it is Serializable, Deserializeable, and Cloneable.
/// 2. Implement Into<Product> (via a From<> impl)
/// 3. Assign it as the create_request type with the #[create_type(CreateProductRequest)]
///    attribute, in the #[derive(Insertable)] implementation.
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct CreateProductRequest {
    name: String,
    description: String,
    price_usd_cents: u64,
}

impl From<CreateProductRequest> for Product {
    fn from(val: CreateProductRequest) -> Self {
        Product {
            id: uuid::Uuid::new_v4(),
            name: val.name,
            description: val.description,
            stripe_price_id: None,
            stripe_product_id: None,
            price_usd_cents: val.price_usd_cents as i64,
            image_metadata: None,
        }
    }
}
pub async fn save_image(
    mut req: Request,
    // prod_id: PathString,
    // image: Image,
    db_images: PostgresDataProvider<ImageMetadata>,
    products: PostgresDataProvider<Product>,
) -> Response {
    // TODO [TECH DEBT]: path_params is a vec, but it should really be a map. Need to be able to look up by param name (parsed from route definition)
    let (Some(Ok(product_id)), Ok(image)) = (
        req.path_params.pop().map(|id| uuid::Uuid::parse_str(&id)),
        <Image as FromRequest>::from(req),
    ) else {
        log::error!("Missing valid request");
        return Response::bad_request();
    };
    let filename = format!("./downloaded_images/{}", &image.metadata.key);
    let metadata = match db_images.create(image.metadata).await {
        Ok(result) => result,
        Err(e) => {
            log::error!("Error saving image to DB: {:?}", e);
            return Response::internal_server_error();
        },
    };
    let Ok(Some(mut prod)) = products.get(|p| p.id.eq(product_id)).await else {
        return Response::not_found();
    };
    std::fs::write(filename, image.bytes).unwrap();
    prod.image_metadata = Some(metadata);
    products.update(&prod).await.unwrap();
    Response::ok()
}

// TODO: Find a way to move this to the DataProvider. Using the standard From<> trait makes this not really work.
pub async fn create_product(
    req: CreateProductRequest,
    products: PostgresDataProvider<Product>,
    ServerData(stripe_client): ServerData<stripe::Client>,
) -> Response {
    let mut product = products.create(req).await.unwrap();
    if create_stripe_product(&mut product, stripe_client).await.is_err() {
        return Response::internal_server_error();
    }
    if products.update(&product).await.is_err() {
        return Response::internal_server_error();
    };
    product.into_response()
}

///  Creates a new product on Stripe. Requires the secret be configured as an ENV variable.
async fn create_stripe_product(
    product: &mut Product,
    stripe_client: stripe::Client,
) -> Result<(), StripeError> {
    let id = product.id.to_string();

    let stripe_product = stripe::CreateProduct {
        id: Some(&id),
        name: product.name.as_str(),
        active: Some(true),
        default_price_data: Some(stripe::CreateProductDefaultPriceData {
            currency: stripe::Currency::USD,
            tax_behavior: Some(stripe::CreateProductDefaultPriceDataTaxBehavior::Exclusive),
            unit_amount: Some(product.price_usd_cents),
            ..Default::default()
        }),
        images: None,
        description: None,
        expand: &[],
        // images: Some(Box::new(self.image_urls.clone())),
        metadata: None,
        package_dimensions: None,
        shippable: Some(true),
        statement_descriptor: None,
        tax_code: None,
        unit_label: None,
        url: None,
        features: None,
        type_: None,
    };

    let stripe_product = stripe::Product::create(&stripe_client, stripe_product).await;
    let stripe_product = dbg!(stripe_product)?;
    product.stripe_product_id = Some(stripe_product.id.to_string());

    match &dbg!(stripe_product).default_price {
        Some(price) => {
            product.stripe_price_id = Some(price.id().to_string());
            Ok(())
        },
        None => {
            Err(stripe::StripeError::ClientError("Failed to create price at Stripe.".to_string()))
        },
    }
}
