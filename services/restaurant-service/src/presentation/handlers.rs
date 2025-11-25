use actix_web::{web, HttpRequest, HttpResponse};
use error_handling::{AppError, AppResult};
use models::restaurant::*;
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::application::RestaurantService;

// ==================== Helper Functions ====================

/// Extract owner_id from X-Owner-Id header
fn get_owner_id(req: &HttpRequest) -> AppResult<Uuid> {
    req.headers()
        .get("X-Owner-Id")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<Uuid>().ok())
        .ok_or_else(|| AppError::BadRequest("Missing or invalid X-Owner-Id header".to_string()))
}

// ==================== Restaurant Handlers ====================

/// Create a new restaurant
pub async fn create_restaurant(
    service: web::Data<tokio::sync::Mutex<RestaurantService>>,
    request: web::Json<CreateRestaurantRequest>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    request.validate()?;
    let owner_id = get_owner_id(&req)?;

    let mut service = service.lock().await;
    let restaurant = service.create_restaurant(owner_id, request.into_inner()).await?;

    Ok(HttpResponse::Created().json(restaurant))
}

/// Get restaurant by ID
pub async fn get_restaurant(
    service: web::Data<tokio::sync::Mutex<RestaurantService>>,
    path: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    let restaurant_id = path.into_inner();
    
    let service = service.lock().await;
    let restaurant = service.get_restaurant(restaurant_id).await?;
    
    Ok(HttpResponse::Ok().json(restaurant))
}

/// Get all restaurants for an owner
pub async fn get_owner_restaurants(
    service: web::Data<tokio::sync::Mutex<RestaurantService>>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let owner_id = get_owner_id(&req)?;

    let service = service.lock().await;
    let restaurants = service.get_owner_restaurants(owner_id).await?;

    Ok(HttpResponse::Ok().json(restaurants))
}

/// Update restaurant
pub async fn update_restaurant(
    service: web::Data<tokio::sync::Mutex<RestaurantService>>,
    path: web::Path<Uuid>,
    request: web::Json<CreateRestaurantRequest>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    request.validate()?;
    let owner_id = get_owner_id(&req)?;
    let restaurant_id = path.into_inner();

    let mut service = service.lock().await;
    let restaurant = service.update_restaurant(restaurant_id, owner_id, request.into_inner()).await?;

    Ok(HttpResponse::Ok().json(restaurant))
}

/// Delete restaurant
pub async fn delete_restaurant(
    service: web::Data<tokio::sync::Mutex<RestaurantService>>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let owner_id = get_owner_id(&req)?;
    let restaurant_id = path.into_inner();

    let mut service = service.lock().await;
    service.delete_restaurant(restaurant_id, owner_id).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Restaurant deleted successfully"
    })))
}

// ==================== Category Handlers ====================

/// Create a new food category
pub async fn create_category(
    service: web::Data<tokio::sync::Mutex<RestaurantService>>,
    path: web::Path<Uuid>,
    request: web::Json<CreateFoodCategoryRequest>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    request.validate()?;
    let owner_id = get_owner_id(&req)?;
    let restaurant_id = path.into_inner();

    let mut service = service.lock().await;
    let category = service.create_category(restaurant_id, owner_id, request.into_inner()).await?;

    Ok(HttpResponse::Created().json(category))
}

#[derive(Deserialize)]
pub struct CategoryPath {
    pub restaurant_id: Uuid,
    pub category_id: Uuid,
}

/// Update food category
pub async fn update_category(
    service: web::Data<tokio::sync::Mutex<RestaurantService>>,
    path: web::Path<CategoryPath>,
    request: web::Json<CreateFoodCategoryRequest>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    request.validate()?;
    let owner_id = get_owner_id(&req)?;
    let path = path.into_inner();

    let mut service = service.lock().await;
    let category = service.update_category(
        path.restaurant_id,
        path.category_id,
        owner_id,
        request.into_inner(),
    ).await?;

    Ok(HttpResponse::Ok().json(category))
}

/// Delete food category
pub async fn delete_category(
    service: web::Data<tokio::sync::Mutex<RestaurantService>>,
    path: web::Path<CategoryPath>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let owner_id = get_owner_id(&req)?;
    let path = path.into_inner();

    let mut service = service.lock().await;
    service.delete_category(path.restaurant_id, path.category_id, owner_id).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Category deleted successfully"
    })))
}

// ==================== Food Handlers ====================

/// Create a new food item
pub async fn create_food(
    service: web::Data<tokio::sync::Mutex<RestaurantService>>,
    path: web::Path<Uuid>,
    request: web::Json<CreateFoodRequest>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    request.validate()?;
    let owner_id = get_owner_id(&req)?;
    let restaurant_id = path.into_inner();

    let mut service = service.lock().await;
    let food = service.create_food(restaurant_id, owner_id, request.into_inner()).await?;

    Ok(HttpResponse::Created().json(food))
}

#[derive(Deserialize)]
pub struct FoodPath {
    pub restaurant_id: Uuid,
    pub food_id: Uuid,
}

/// Update food item
pub async fn update_food(
    service: web::Data<tokio::sync::Mutex<RestaurantService>>,
    path: web::Path<FoodPath>,
    request: web::Json<CreateFoodRequest>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    request.validate()?;
    let owner_id = get_owner_id(&req)?;
    let path = path.into_inner();

    let mut service = service.lock().await;
    let food = service.update_food(
        path.restaurant_id,
        path.food_id,
        owner_id,
        request.into_inner(),
    ).await?;

    Ok(HttpResponse::Ok().json(food))
}

/// Delete food item
pub async fn delete_food(
    service: web::Data<tokio::sync::Mutex<RestaurantService>>,
    path: web::Path<FoodPath>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let owner_id = get_owner_id(&req)?;
    let path = path.into_inner();

    let mut service = service.lock().await;
    service.delete_food(path.restaurant_id, path.food_id, owner_id).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Food item deleted successfully"
    })))
}

// ==================== Menu Handlers ====================

/// Get full menu for a restaurant
pub async fn get_menu(
    service: web::Data<tokio::sync::Mutex<RestaurantService>>,
    path: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    let restaurant_id = path.into_inner();

    let mut service = service.lock().await;
    let menu = service.get_menu(restaurant_id).await?;

    Ok(HttpResponse::Ok().json(menu))
}

// ==================== Internal Handlers ====================

#[derive(Deserialize)]
pub struct FoodIdsQuery {
    pub ids: String, // Comma-separated UUIDs
}

/// Get foods by IDs (internal endpoint for Order Service)
pub async fn get_foods_by_ids(
    service: web::Data<tokio::sync::Mutex<RestaurantService>>,
    query: web::Query<FoodIdsQuery>,
) -> AppResult<HttpResponse> {
    // Parse comma-separated UUIDs
    let ids: Result<Vec<Uuid>, _> = query
        .ids
        .split(',')
        .map(|s| s.trim().parse::<Uuid>())
        .collect();

    let ids = ids.map_err(|_| error_handling::AppError::BadRequest("Invalid UUID format".to_string()))?;

    let service = service.lock().await;
    let foods = service.get_foods_by_ids(ids).await?;

    Ok(HttpResponse::Ok().json(foods))
}

// ==================== Health Check ====================

pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "restaurant-service"
    }))
}
