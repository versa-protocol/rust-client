use serde_json::Value;
use versa::protocol::{misuse::MisuseCode, WebhookEventType};

const SCHEMA_URL: &'static str = "https://raw.githubusercontent.com/versa-protocol/schema";

pub async fn validate(event: &WebhookEventType, data: &Value) -> Result<(), (MisuseCode, String)> {
  let schema_version = match data.get("schema_version") {
    Some(val) => match val.as_str() {
      Some(version) => version,
      None => {
        return Err((
          MisuseCode::SchemaVersionInvalid,
          format!("Invalid schema_version: {}", val),
        ))
      }
    },
    None => {
      return Err((
        MisuseCode::SchemaValidationFailed,
        "Missing schema_version".to_string(),
      ))
    }
  };
  let schema_url = format!(
    "{}/{}/data/{}.schema.json",
    SCHEMA_URL,
    schema_version,
    event.to_string()
  );

  // get schema from URL using reqwest and turn into serde Value
  let schema: Value = match match reqwest::get(&schema_url).await {
    Ok(res) => res,
    Err(_) => {
      return Err((
        MisuseCode::SchemaVersionInvalid,
        format!("Invalid schema_version: {}", schema_version),
      ))
    }
  }
  .json()
  .await
  {
    Ok(val) => val,
    Err(_) => {
      return Err((
        MisuseCode::SchemaVersionInvalid,
        format!("Invalid schema_version: {}", schema_version),
      ))
    }
  };

  match jsonschema::validate(&schema, data) {
    Ok(_) => Ok(()),
    Err(e) => {
      let error_message = format!("Schema validation failed: {}", e);
      Err((MisuseCode::SchemaValidationFailed, error_message))
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_validation_should_succeed() {
    let data = serde_json::json!({
      "schema_version": "1.4.0",
      "header": {
        "invoice_number": "auth_1MzFN1K8F4fqH0lBmFq8CjbU",
        "currency": "usd",
        "total": 2212,
        "subtotal": 1780,
        "paid": 2212,
        "invoiced_at": 1713295619,
        "mcc": null,
        "third_party": null,
        "customer": null,
        "location": null,
        "invoice_asset_id": null,
        "receipt_asset_id": null
      },
      "itemization": {
        "general": {
          "line_items": [
            {
              "description": "Widget",
              "subtotal": 1780,
              "quantity": 10,
              "unit_cost": 178,
              "unit": null,
              "taxes": [
                {
                  "amount": 432,
                  "rate": 0.0875,
                  "name": "GST"
                }
              ],
              "metadata": [],
              "product_image": null,
              "date": null,
              "url": null,
              "adjustments": []
            }
          ],
          "invoice_level_adjustments": []
        },
        "lodging": null,
        "ecommerce": null,
        "car_rental": null,
        "transit_route": null,
        "subscription": null,
        "flight": null
      },
      "actions": [],
      "payments": []
    });

    assert!(validate(&WebhookEventType::Receipt, &data).await.is_ok());
  }

  #[tokio::test]
  async fn test_validation_of_incomplete_receipt_should_fail() {
    let data = serde_json::json!({
      "schema_version": "1.10.0",
      "header": {
        "invoice_number": "auth_1MzFN1K8F4fqH0lBmFq8CjbU",
        "currency": "usd",
        "subtotal": 1780,
        "paid": 2212,
        "invoiced_at": 1713295619
      },
      "itemization": {
        "general": {
          "line_items": [
            {
              "description": "Widget",
              "quantity": 10,
              "adjustments": []
            }
          ],
          "invoice_level_adjustments": []
        }
      },
    });

    let Err((code, msg)) = validate(&WebhookEventType::Receipt, &data).await else {
      panic!("This test validation case should fail");
    };
    assert_eq!(code, MisuseCode::SchemaValidationFailed);
    assert_eq!(
      msg,
      "Schema validation failed: \"total\" is a required property"
    );
  }

  #[tokio::test]
  async fn test_validation_of_outdated_schema_version_should_fail() {
    let data = serde_json::json!({
      "schema_version": "1.0",
      "header": {
        "invoice_number": "auth_1MzFN1K8F4fqH0lBmFq8CjbU",
        "currency": "usd",
        "subtotal": 1780,
        "paid": 2212,
        "invoiced_at": 1713295619
      },
      "itemization": {
        "general": {
          "line_items": [
            {
              "description": "Widget",
              "quantity": 10,
              "adjustments": []
            }
          ],
          "invoice_level_adjustments": []
        }
      },
    });

    let Err((code, msg)) = validate(&WebhookEventType::Receipt, &data).await else {
      panic!("This test validation case should fail");
    };
    assert_eq!(code, MisuseCode::SchemaVersionInvalid);
    assert_eq!(msg, "Invalid schema_version: 1.0");
  }

  #[tokio::test]
  async fn test_validation_of_itinerary_with_payment_should_fail() {
    let data = serde_json::json!({
      "schema_version": "1.10.0",
      "header": {
        "subtotal": 1780,
      },
      "itemization": {
        "general": {
          "line_items": [
            {
              "description": "Widget",
              "quantity": 10,
              "adjustments": []
            }
          ],
          "invoice_level_adjustments": []
        }
      },
    });

    let Err((code, msg)) = validate(&WebhookEventType::Itinerary, &data).await else {
      panic!("This test validation case should fail");
    };
    assert_eq!(code, MisuseCode::SchemaValidationFailed);
    assert_eq!(
      msg,
      "Schema validation failed: Additional properties are not allowed ('subtotal' was unexpected)"
    );
  }
}
