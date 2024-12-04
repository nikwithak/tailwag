use crate::stripe_integration::stripe_event;
use tailwag_macros::Display;
use uuid::Uuid;

use tailwag;

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
    pub(crate) customer_name: String,
    pub(crate) customer_email: String,
    pub(crate) status: String, // TODO: Pipe enums through here.
    pub(crate) stripe_session_id: String,
    pub(crate) confirmation_email_sent: bool,
    // #[no_filter]
    #[create_ignore]
    #[no_filter]
    #[no_form]
    // TODO: This is breaking the form definition - I need to figure out how to represent nested structs EVERYWHERE
    pub(crate) order_amount: Option<OrderAmount>,
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
pub enum ShopOrderStatus {
    Created,
    Canceled,
    Paid,
    Shipped,
    Delivered,
    Completed,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct CartItem {
    pub(crate) id: Uuid,
    pub(crate) quantity: usize,
    // TODO:
    // customizations: Vec<Customization>,
}

pub mod checkout {
    use std::collections::HashMap;

    use crate::{CartItem, Product, ShopOrder};
    use stripe::CreateCheckoutSessionLineItems;
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
        log::debug!("Got a request: {:?}", req);
        let mut stripe_line_items = Vec::new();
        for lineitem in req.cart_items {
            let product = products
                .get(
                    move |filter| filter.id.eq(lineitem.id), // .eq(i.product_id.clone())
                )
                .await?
                .ok_or(tailwag_web_service::Error::BadRequest("Invalid product ID".into()))?;

            let stripe_item = stripe::CreateCheckoutSessionLineItems {
                adjustable_quantity: None,
                dynamic_tax_rates: None,
                price: product.stripe_price_id.clone(),
                price_data: None,
                quantity: Some(lineitem.quantity as u64), // TODO: Actually get the quantity.
                tax_rates: None,
            };
            stripe_line_items.push(stripe_item);
        }

        // impl From<&Product> for stripe::CreateCheckoutSessionLineItems {
        //     fn from(val: &Product) -> Self {
        //     }
        // }

        type OrderCreateRequest =
            <PostgresDataProvider<ShopOrder> as DataProvider<ShopOrder>>::CreateRequest;
        let order = OrderCreateRequest {
            ..Default::default()
        };
        let order = orders.create(order).await?;

        let url = create_stripe_session(order, stripe_line_items, &stripe_client)
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
        lineitems: Vec<CreateCheckoutSessionLineItems>,
        stripe_client: &stripe::Client,
    ) -> stripe::CheckoutSession {
        let order_id = &order.id.to_string();
        let success_url = format!("http://localhost:3000/order/{}", &order.id); // TODO: Unhardcode this. Should it be part of the request from client?
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
            line_items: Some(lineitems),
            ..Default::default()
        };

        stripe::CheckoutSession::create(stripe_client, params).await.unwrap()
    }
}
