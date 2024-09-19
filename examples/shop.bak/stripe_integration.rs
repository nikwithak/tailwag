use serde::{Deserialize, Serialize};
use stripe::{CheckoutSessionPaymentStatus, Event, EventObject};
use tailwag_orm::{
    data_manager::{traits::DataProvider, PostgresDataProvider},
    queries::filterable_types::FilterEq,
};
use tailwag_utils::email::sendgrid::SendGridEmailClient;
use tailwag_web_service::{
    application::http::route::{IntoResponse, Request, Response},
    tasks::TaskScheduler,
};

use crate::{OrderAmount, ShopOrder, ShopOrderStatus};

pub type StripeSecret = String;
pub async fn stripe_event(
    request: Request,
    _data_providers: tailwag_orm::data_definition::exp_data_system::DataSystem,
    mut task_queuer: TaskScheduler,
) -> impl IntoResponse {
    let webhook_secret =
        std::env::var("STRIPE_WEBHOOK_SECRET").expect("STRIPE_WEBHOOK_SECRET missing");

    // Verify / Decode the stripe event
    let Some(stripe_signature) = request.headers.get("stripe-signature").cloned() else {
        return Response::not_found();
    };
    let tailwag_web_service::application::http::route::HttpBody::Json(body) = request.body else {
        return Response::bad_request();
    };
    let event = match stripe::Webhook::construct_event(&body, &stripe_signature, &webhook_secret) {
        Ok(event) => event,
        Err(err) => {
            log::error!("[STRIPE] Failed to unpack stripe event: {}", err.to_string());
            return Response::bad_request();
        },
    };
    let event_id = event.id.clone();
    log::debug!("[STRIPE] Received event id: {event_id}");

    // Send the event to our event processor.
    // TODO [TECH DEBT]: This can be one line if I just figure out add '?' support to Response / IntoResponse.
    let Ok(ticket) = task_queuer.enqueue(ProcessStripeEvent {
        event,
    }) else {
        log::error!("[TICKET CREATE FAILED] Failed to send task to handler.");
        return Response::internal_server_error();
    };
    log::info!("Created ticket: {}", &ticket.id());

    Response::ok()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProcessStripeEvent {
    pub event: Event,
}
pub async fn handle_stripe_event(
    event: ProcessStripeEvent,
    orders: PostgresDataProvider<ShopOrder>,
) -> String {
    log::info!("[STRIPE] Received event in task queue: {:?}", event);
    process_event(&event.event, orders).await.unwrap();
    "Finished.".to_string()
}

async fn process_checkout_session_completed_event(
    event: &stripe::Event,
    orders: impl DataProvider<ShopOrder>,
) -> Result<(), tailwag_web_service::Error> {
    let EventObject::CheckoutSession(session) = &event.data.object else {
        Err("Invalid checkout session received.")?
    };
    let Some(order_id) = session
        .client_reference_id
        .as_ref()
        .and_then(|oid| uuid::Uuid::parse_str(oid).ok())
    else {
        Err("Invalid order ID received")?
    };
    let Ok(Some(mut order)) = orders.get(|o| o.id.eq(order_id)).await else {
        Err("Could not find order in DB")?
    };

    order.stripe_session_id = session.id.to_string();
    match session.payment_status {
        CheckoutSessionPaymentStatus::Paid => order.status = ShopOrderStatus::Paid.to_string(),
        CheckoutSessionPaymentStatus::Unpaid => {
            order.status = ShopOrderStatus::Canceled.to_string()
        },
        CheckoutSessionPaymentStatus::NoPaymentRequired => {
            order.status = ShopOrderStatus::Paid.to_string()
        },
    }

    if let Some(customer) = &session.customer_details {
        order.customer_name = customer.name.as_ref().map_or("".to_string(), |s| s.to_owned());
        order.customer_email = customer.email.as_ref().map_or("".to_string(), |s| s.to_owned());
    };

    // if let Some(stripe_shipping) = &session.shipping {
    //     order.shipping_detail = Some(ShippingInfo::from(&**stripe_shipping));
    // }
    // order.customer_phone = session.customer_details.as_ref().map_or(None,
    // |detail| { detail.phone.map_or(None, |p|
    // Some(*p.clone()) });
    // if let Some(customer) = session.customer {
    //     match *customer {
    //         stripe::Expandable::Object(customer) => {
    //             order.customer_name = customer.name.map(|n| *n.clone());
    //         },
    //         stripe::Expandable::Id(id) => {
    //             order.customer_name =  Some(id.to_string());
    //             log::info!("Order processed for customer_id {}", id);
    //         }
    //     }
    // } else {
    //     log::info!("No customer found");
    // }

    let order_amount = OrderAmount::from(session);
    order.order_amount = Some(order_amount);

    // Send emails if it hasn't already been sent
    if !order.confirmation_email_sent {
        // TODO: Externalize content
        let sendgrid_client = SendGridEmailClient::from_env()?;
        sendgrid_client
            .send_email(
                // TODO [HARDCODING] [CONFIG]: Read this from config
                // &order.customer_email,
                "nwakg@pm.me", // Harddcoding my email for testing, to avoid sending emails randomly to fake test emails
                "Your Tailwag Order",
                r#"Your order has been successfully placed! Payment received, etc. etc.

Thanks for ordering.

Wof wof!
"#,
            )
            .await?;
        order.confirmation_email_sent = true;

        //     // Send email to Business
        //     match send_email(
        //         "nik@tonguetail.com",
        //         "You have a new order!!",
        //         "Visit https://beta.scrapplique.com/manage/orders to view it",
        //     )
        //     .await
        //     {
        //         Err(e) => {
        //             log::error!("Failed to send order confirmation email {}", e);
        //         },
        //         _ => log::info!("Send email for order id {}", &order.id),
        //     };
        //     order.confirmation_email_sent = true;
    }

    // if orders.update(&order).await.is_err() {
    //     log::error!("Failed to update order {}", &order_id);
    // };
    orders.update(&order).await.unwrap();
    log::info!("Order {order_id} status updated to {}", order_id);
    Ok(())
}
pub async fn process_event(
    event: &stripe::Event,
    orders: impl DataProvider<ShopOrder>,
) -> Result<(), tailwag_web_service::Error> {
    let stripe::Event {
        type_,
        ..
    } = &event;

    match type_ {
        stripe::EventType::CheckoutSessionCompleted => {
            process_checkout_session_completed_event(event, orders).await
        },
        _ => {
            log::info!("Ignoring webhook event {}", serde_json::to_string(type_)?);
            Ok(())
        },
    }
}
