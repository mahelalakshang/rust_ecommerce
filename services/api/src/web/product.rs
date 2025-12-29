use std::sync::Arc;

use axum::{Extension, Json, extract::State};

use crate::{
    config::Config,
    dtos::{CreateProductRequest, ProductResponse},
    error::AppError,
    model::Product,
};
use uuid::Uuid;

pub async fn create_product(
    State(state): State<Arc<Config>>,
    Extension(user_id): Extension<Uuid>,
    Json(payload): Json<CreateProductRequest>,
) -> Result<Json<ProductResponse>, AppError> {
    let product = sqlx::query_as::<_, Product>(
        "INSERT INTO products (user_id, category_id, name, description, price, stock_quantity) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
    )
    .bind(&user_id)
    .bind(&payload.category_id)
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(&payload.price)
    .bind(&payload.stock_quantity)
    .fetch_one(&state.db_pool)
    .await?;

    // Send notification to the notification service
    let notification_result = crate::grpc_client::send_product_notification(
        &user_id.to_string(),
        &product.name,
    )
    .await;

    // Log if notification fails, but don't fail the product creation
    if let Err(e) = notification_result {
        tracing::warn!("Failed to send notification: {}", e);
    }

    Ok(Json(ProductResponse {
        id: product.id,
        category_id: product.category_id,
        name: product.name.clone(),
        description: product.description,
        price: product.price,
        stock_quantity: product.stock_quantity,
    }))
}
