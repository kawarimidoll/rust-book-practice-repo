use crate::model::{
    checkout::{
        event::{CreateCheckout, UpdateReturned},
        Checkout,
    },
    id::{BookId, UserId},
};
use async_trait::async_trait;
use shared::error::AppResult;

#[mockall::automock]
#[async_trait]
pub trait CheckoutRepository: Send + Sync {
    // create checkout -> 貸出
    async fn create(&self, event: CreateCheckout) -> AppResult<()>;
    // update returned_at -> 返却
    async fn update_returned(&self, event: UpdateReturned) -> AppResult<()>;
    // すべての貸出中の情報を取得
    async fn find_unreturned_all(&self) -> AppResult<Vec<Checkout>>;
    // 特定のユーザーの貸出中の情報を取得
    async fn find_unreturned_by_user_id(&self, user_id: UserId) -> AppResult<Vec<Checkout>>;
    // 特定の蔵書の貸出履歴を取得
    async fn find_history_by_book_id(&self, book_id: BookId) -> AppResult<Vec<Checkout>>;
}
