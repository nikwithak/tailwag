use serde::{Deserialize, Serialize};
use stripe_integration::stripe_event;
use tailwag_macros::Display;
use tailwag_web_service::{
    application::http::route::{IntoResponse, RequestContext, Response},
    auth::gateway::{self, extract_session, Session},
    extras::image_upload::{self, ImageMetadata},
    tasks::TaskScheduler,
};
use uuid::Uuid;
mod product;
pub use product::*;

pub mod stripe_integration;

#[tokio::main]
async fn main() {
    tailwag_web_service::application::WebService::builder("My Shop Service")
        .with_middleware(extract_session)
        .post_public("/login", gateway::login)
        .post_public("/register", gateway::register)
        .with_resource::<Product>() // TODO- public GET with filtering)
        .with_resource::<ShopOrder>() // TODO - public POST, restricted GET (to specific customer, via email)
        .with_resource::<OrderAmount>() // TODO - Needed to make sure the tables get created. TODO: Auto-create all direct dependent tables automatically in the ORM
        .with_resource::<ImageMetadata>() // TODO - Needed to make sure the tables get created. TODO: Auto-create all direct dependent tables automatically in the ORM
        .get_public("/status", || "Hello".into_response())
        .post_public("/checkout", checkout::checkout) // TODO
        .post_public("/email", email_webhook)
        .get_public("/image/{filename}", image_upload::load_image)
        .with_server_data(stripe::Client::new(
            std::env::var("STRIPE_API_KEY").expect("STRIPE_API_KEY is missing from env."), // TODO: Move to a 'config' automation / macro.
        ))
        .with_task(stripe_integration::handle_stripe_event)
        .with_task(send_email)
        .with_static_files()
        .build_service()
        .run()
        .await
        .unwrap();
}

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
#[actions(stripe_event)]
#[post(checkout::checkout)] // This should be the only way to create an order (for now)
pub struct ShopOrder {
    id: Uuid,
    customer_name: String,
    customer_email: String,
    status: String, // TODO: Pipe enums through here.
    stripe_session_id: String,
    confirmation_email_sent: bool,
    // #[no_filter]
    #[create_ignore]
    #[no_filter]
    #[no_form]
    // TODO: This is breaking the form definition - I need to figure out how to represent nested structs EVERYWHERE
    order_amount: Option<OrderAmount>,
    // line_items: Vec<Product>,
    // TODO: once I implement flatten / other types, this can auto-expand to:
    // amount_subtotal: i64,
    // amount_tax: i64,
    // amount_shipping: i64,
    // amount_discount: i64,
    // amount_total: i64,
}

#[derive(
    Clone,
    Debug,
    serde::Deserialize,
    serde::Serialize,
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
pub struct OrderAmount {
    id: Uuid,
    subtotal_amount: i64,
    tax_amount: i64,
    shipping_amount: i64,
    discount_amount: i64,
    total_amount: i64,
}
impl Default for OrderAmount {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            subtotal_amount: Default::default(),
            tax_amount: Default::default(),
            shipping_amount: Default::default(),
            discount_amount: Default::default(),
            total_amount: Default::default(),
        }
    }
}

impl From<&stripe::CheckoutSession> for OrderAmount {
    fn from(stripe_session: &stripe::CheckoutSession) -> Self {
        let subtotal_amount = stripe_session.amount_subtotal.as_ref().map_or(0, |b| *b);
        let total_amount = stripe_session.amount_total.as_ref().map_or(0, |b| *b);
        let (tax_amount, shipping_amount, discount_amount) =
            stripe_session.total_details.as_ref().map_or((0, 0, 0), |amounts| {
                (
                    amounts.amount_tax,
                    amounts.amount_shipping.as_ref().map_or(0, |b| *b),
                    amounts.amount_discount,
                )
            });

        Self {
            id: Uuid::new_v4(),
            subtotal_amount,
            tax_amount,
            shipping_amount,
            discount_amount,
            total_amount,
        }
    }
}

#[derive(Display)]
#[allow(unused)]
enum ShopOrderStatus {
    Created,
    Canceled,
    Paid,
    Shipped,
    Delivered,
    Completed,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct CartItem {
    id: Uuid,
    quantity: usize,
    // TODO:
    // customizations: Vec<Customization>,
}

pub mod checkout {
    use std::collections::HashMap;

    use crate::{CartItem, Product, ShopOrder};
    use tailwag_orm::{
        data_manager::{traits::DataProvider, PostgresDataProvider},
        queries::filterable_types::FilterEq,
    };
    use tailwag_web_service::{
        application::http::route::{IntoResponse, Response, ServerData},
        Error,
    };

    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    pub struct CheckoutRequest {
        cart_items: Vec<CartItem>,
        customer_name: Option<String>,
        customer_email: Option<String>,
    }
    pub async fn checkout(
        req: CheckoutRequest,
        products: PostgresDataProvider<Product>,
        orders: PostgresDataProvider<ShopOrder>,
        stripe_client: ServerData<stripe::Client>,
    ) -> Result<Response, tailwag_web_service::Error> {
        let products_fut = req.cart_items.iter().map(|i| {
            products.get(
                move |filter| filter.id.eq(i.id), // .eq(i.product_id.clone())
            )
        });
        let mut order_products = Vec::new();
        for product in products_fut {
            order_products.push(product.await.unwrap().unwrap())
        }

        type OrderCreateRequest =
            <PostgresDataProvider<ShopOrder> as DataProvider<ShopOrder>>::CreateRequest;
        let order = OrderCreateRequest {
            ..Default::default()
        };
        let order = orders.create(order).await?;
        // let Ok(order) = orders.create(order).await else {
        //     log::error!("Failed to create order");
        //     // TODO: Figure out how to consume the ? operator here. Writing this every time is annoying.
        //     return Response::internal_server_error();
        // };

        log::debug!("Got a request: {:?}", req);
        let url = create_stripe_session(order, order_products, &stripe_client)
            .await
            .url
            .ok_or(Error::InternalServerError("Failed to create stripe session.".into()))?;

        // Response::redirect_see_other(&url)
        Ok(vec![("payment_url", url)]
            .into_iter()
            .collect::<HashMap<&str, String>>()
            .into_response())
    }

    async fn create_stripe_session(
        order: ShopOrder,
        products: Vec<Product>,
        stripe_client: &stripe::Client,
    ) -> stripe::CheckoutSession {
        let order_id = &order.id.to_string();
        let success_url = format!("http://localhost:3000/order/{}", &order.id);
        let params = stripe::CreateCheckoutSession {
            success_url: Some(&success_url), // TODO: Configure this
            // customer_email: Some(&order.customer_email),
            shipping_address_collection: Some(
                stripe::CreateCheckoutSessionShippingAddressCollection {
                    allowed_countries: vec![
                        stripe::CreateCheckoutSessionShippingAddressCollectionAllowedCountries::Us,
                    ],
                },
            ),
            automatic_tax: Some(stripe::CreateCheckoutSessionAutomaticTax {
                enabled: true,
                liability: None,
            }),
            payment_intent_data: Some(stripe::CreateCheckoutSessionPaymentIntentData {
                receipt_email: None,
                ..Default::default()
            }),
            client_reference_id: Some(order_id),
            mode: Some(stripe::CheckoutSessionMode::Payment),
            line_items: Some(products.iter().map(|li| li.into()).collect()),
            ..Default::default()
        };

        stripe::CheckoutSession::create(stripe_client, params).await.unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SendEmailEvent {
    pub subject: String,
    pub body: String,
    pub recipient: String,
}

pub async fn email_webhook(
    request: SendEmailEvent,
    _data_providers: tailwag_orm::data_definition::exp_data_system::DataSystem,
    mut task_queuer: TaskScheduler,
    ctx: RequestContext,
) -> impl IntoResponse {
    if ctx.get_request_data::<Session>().is_none() {
        return Response::unauthorized();
    }
    if task_queuer.enqueue(request).is_ok() {
        Response::ok()
    } else {
        Response::internal_server_error()
    }
}

pub async fn send_email(event: SendEmailEvent) {
    let SendEmailEvent {
        subject,
        body,
        recipient,
    } = event;
    let client = tailwag_utils::email::sendgrid::SendGridEmailClient::from_env().unwrap();
    client.send_email(&recipient, &subject, &body).await.unwrap();
}
