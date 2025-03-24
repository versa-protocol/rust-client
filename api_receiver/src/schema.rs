use serde_json::Value;
use versa::protocol::{misuse::MisuseCode, WebhookEventType};

pub async fn validate(event: &WebhookEventType, data: &Value) -> Result<(), (MisuseCode, String)> {
  let validator = versa::schema::validator::Validator::new().allow_remote_lookup(true);
  validator.validate(event, data).await
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_validation_of_latest_schema_version_should_succeed() {
    let data = serde_json::json!({
      "schema_version": "1.10.0",
      "header": {
        "invoice_number": "1MzFN1K8F4fqH0lBmFq8CjbU",
        "currency": "usd",
        "total": 140040,
        "subtotal": 121862,
        "paid": 140040,
        "invoiced_at": 1713295619,
        "mcc": null,
        "third_party": null,
        "customer": null,
        "location": null,
        "receipt_asset_id": null,
        "invoice_asset_id": null
      },
      "itemization": {
        "general": null,
        "lodging": null,
        "ecommerce": null,
        "car_rental": null,
        "transit_route": null,
        "subscription": null,
        "flight": {
          "tickets": [
            {
              "taxes": [],
              "segments": [
                {
                  "fare": 12186,
                  "departure_airport_code": "MSP",
                  "arrival_airport_code": "GFK",
                  "aircraft_type": "B39M",
                  "departure_at": 1713206492,
                  "arrival_at": 1713226492,
                  "departure_tz": "America/Chicago",
                  "arrival_tz": "America/Chicago",
                  "flight_number": "DL4656",
                  "class_of_service": "k",
                  "seat": "10A",
                  "taxes": [
                    {
                      "amount": 914,
                      "rate": 0.075,
                      "name": "US Transportation Tax"
                    },
                    {
                      "amount": 224,
                      "rate": null,
                      "name": "US September 11th Security Fee"
                    },
                    {
                      "amount": 360,
                      "rate": null,
                      "name": "US Passenger Facility Charge"
                    },
                    {
                      "amount": 400,
                      "rate": null,
                      "name": "US Flight Segment Tax"
                    }
                  ],
                  "adjustments": [],
                  "metadata": []
                },
              ],
              "number": "0062698215636",
              "record_locator": "CU9GEF",
              "passenger": "Susy Smith",
              "metadata": [{ "key": "AAdvantage #", "value": "TH4700" }]
            },
            {
              "taxes": [],
              "segments": [
                {
                  "fare": 12186,
                  "departure_airport_code": "MSP",
                  "arrival_airport_code": "GFK",
                  "aircraft_type": "B39M",
                  "departure_at": 1713206492,
                  "arrival_at": 1713226492,
                  "departure_tz": "America/Chicago",
                  "arrival_tz": "America/Chicago",
                  "flight_number": "DL4656",
                  "class_of_service": "k",
                  "seat": "11A",
                  "taxes": [
                    {
                      "amount": 914,
                      "rate": 0.075,
                      "name": "US Transportation Tax"
                    },
                    {
                      "amount": 224,
                      "rate": null,
                      "name": "US September 11th Security Fee"
                    },
                    {
                      "amount": 360,
                      "rate": null,
                      "name": "US Passenger Facility Charge"
                    },
                    {
                      "amount": 400,
                      "rate": null,
                      "name": "US Flight Segment Tax"
                    }
                  ],
                  "adjustments": [],
                  "metadata": []
                },
              ],
              "number": "0062698215637",
              "record_locator": "CU9GEF",
              "passenger": "John Smith",
              "metadata": [{ "key": "AAdvantage #", "value": "TH4703" }]
            }
          ],
          "itinerary_locator": "1122337694093",
          "invoice_level_adjustments": []
        }
      },
      "footer": {
        "actions": [],
        "supplemental_text": "You have 24 hours from the time you first buy your ticket to make changes or cancel for a refund if you booked at least 2 days before departure."
      },
      "payments": [
        {
          "amount": 140040,
          "paid_at": 1713295619,
          "payment_type": "card",
          "card_payment": { "last_four": "4886", "network": "mastercard" },
          "ach_payment": null
        }
      ]
    });

    assert!(validate(&WebhookEventType::Receipt, &data).await.is_ok());
  }

  #[tokio::test]
  async fn test_validation_of_outdated_receipt_should_succeed() {
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
