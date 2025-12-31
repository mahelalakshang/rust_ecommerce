use tonic::{transport::Server, Request, Response, Status};

// Include the generated protobuf code
pub mod notification {
    tonic::include_proto!("notification");
}

use notification::{
    notification_service_server::{NotificationService, NotificationServiceServer},
    ProductNotificationRequest, ProductNotificationResponse,
};

#[derive(Debug, Default)]
pub struct NotificationServiceImpl {}

#[tonic::async_trait]
impl NotificationService for NotificationServiceImpl {
    async fn send_product_notification(
        &self,
        request: Request<ProductNotificationRequest>,
    ) -> Result<Response<ProductNotificationResponse>, Status> {
        let req = request.into_inner();

        // Print the received notification data
        println!("📢 NOTIFICATION RECEIVED:");
        println!("   User ID: {}", req.user_id);
        println!("   Product Name: {}", req.name);
        println!("   Username: {}", req.username);
        println!("   Timestamp: {}", chrono::Utc::now());
        println!("---");

        // Return success response
        let response = ProductNotificationResponse {
            success: true,
            message: format!("Notification received for product: {}", req.name),
        };

        Ok(Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let addr = "0.0.0.0:50051".parse()?;
    let notification_service = NotificationServiceImpl::default();

    println!("🚀 Notification Service starting on {}", addr);

    Server::builder()
        .add_service(NotificationServiceServer::new(notification_service))
        .serve(addr)
        .await?;

    Ok(())
}
