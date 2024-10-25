use versa::{
  client::{customer_registration::CustomerRegistration, ClientError},
  protocol::customer_registration::{CustomerReference, HandleType},
};

pub async fn register_customer<T>(
  versa_client: T,
  handle: String,
  handle_type: HandleType,
  receiver_client_id: Option<String>,
) -> Result<(), ClientError>
where
  T: CustomerRegistration,
{
  let customer_reference = CustomerReference {
    handle,
    handle_type,
    receiver_client_id,
  };

  versa_client
    .register_customer_reference(customer_reference)
    .await
}

pub async fn deregister_customer<T>(
  versa_client: T,
  handle: String,
  handle_type: HandleType,
  receiver_client_id: Option<String>,
) -> Result<(), ClientError>
where
  T: CustomerRegistration,
{
  let customer_reference = CustomerReference {
    handle,
    handle_type,
    receiver_client_id,
  };

  versa_client
    .deregister_customer_reference(customer_reference)
    .await
}
