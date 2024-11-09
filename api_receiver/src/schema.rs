use serde_json::Value;
use versa::protocol::misuse::MisuseCode;

const SCHEMA_URL: &'static str = "https://raw.githubusercontent.com/versa-protocol/schema";
const SCHEMA_PATH: &'static str = "data/receipt.schema.json";

pub async fn validate(data: &Value, schema_version: &str) -> Result<(), MisuseCode> {
  let schema_url = format!("{}/{}/{}", SCHEMA_URL, schema_version, SCHEMA_PATH);

  // get schema from URL using reqwest and turn into serde Value
  let schema: Value = match match reqwest::get(&schema_url).await {
    Ok(res) => res,
    Err(_) => return Err(MisuseCode::SchemaVersionInvalid),
  }
  .json()
  .await
  {
    Ok(val) => val,
    Err(_) => return Err(MisuseCode::SchemaVersionInvalid),
  };

  if jsonschema::is_valid(&schema, data) {
    Ok(())
  } else {
    Err(MisuseCode::SchemaValidationFailed)
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
    let schema_version = "1.4.0";

    assert!(validate(&data, schema_version).await.is_ok());
  }

  #[tokio::test]
  async fn test_validation_of_incomplete_receipt_should_fail() {
    let data = serde_json::json!({
      "schema_version": "1.4.0",
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
    let schema_version = "1.4.0";

    assert!(validate(&data, schema_version).await.is_err());
  }
}
