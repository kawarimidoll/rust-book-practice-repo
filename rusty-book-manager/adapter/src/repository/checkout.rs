use crate::database::{
    model::checkout::{CheckoutRow, CheckoutStateRow, ReturnedCheckoutRow},
    ConnectionPool,
};
use async_trait::async_trait;

use derive_new::new;
use kernel::model::checkout::{
    event::{CreateCheckout, UpdateReturned},
    Checkout,
};
use kernel::model::id::{BookId, CheckoutId, UserId};
use kernel::repository::checkout::CheckoutRepository;
use shared::error::{AppError, AppResult};

#[derive(new)]
pub struct CheckoutRepositoryImpl {
    db: ConnectionPool,
}

#[async_trait]
impl CheckoutRepository for CheckoutRepositoryImpl {
    // create checkout -> 貸出
    async fn create(&self, event: CreateCheckout) -> AppResult<()> {
        let mut tx = self.db.begin().await?;

        // トランザクション分離レベルをSERIALIZABLEに設定
        self.set_transaction_serializable(&mut tx).await?;

        // 条件：
        // - 指定の蔵書 ID をもつ蔵書が存在する
        // - その蔵書が貸出中でない
        {
            let res = sqlx::query_as!(
                CheckoutStateRow,
                r#"
                    SELECT
                    b.book_id,
                    c.checkout_id AS "checkout_id?: CheckoutId",
                    NULL AS "user_id?: UserId"
                    FROM books AS b
                    LEFT OUTER JOIN checkouts AS c USING(book_id)
                    WHERE book_id = $1;
                "#,
                event.book_id as _
            )
            .fetch_optional(&mut *tx)
            .await
            .map_err(AppError::SpecificOperationError)?;

            match res {
                // 条件を満たさなければエラーを返す
                None => {
                    return Err(AppError::EntityNotFound(format!(
                        " 書籍（{}）が見つかりませんでした。",
                        event.book_id
                    )))
                }
                Some(CheckoutStateRow {
                    checkout_id: Some(_),
                    ..
                }) => {
                    return Err(AppError::UnprocessableEntity(format!(
                        " 書籍（{}）に対する貸出が既に存在します。",
                        event.book_id
                    )))
                }
                // それ以外は処理続行
                _ => {}
            }
        }

        // create record
        let checkout_id = CheckoutId::new();
        let res = sqlx::query!(
            r#"
                INSERT INTO checkouts
                (checkout_id, book_id, user_id, checked_out_at)
                VALUES ($1, $2, $3, $4)
                ;
            "#,
            checkout_id as _,
            event.book_id as _,
            event.checked_out_by as _,
            event.checked_out_at,
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::SpecificOperationError)?;

        if res.rows_affected() < 1 {
            return Err(AppError::NoRowsAffectedError(
                "No checkout record has been created".into(),
            ));
        }

        tx.commit().await.map_err(AppError::TransactionError)?;

        Ok(())
    }

    // update returned_at -> 返却
    async fn update_returned(&self, event: UpdateReturned) -> AppResult<()> {
        let mut tx = self.db.begin().await?;

        // トランザクション分離レベルをSERIALIZABLEに設定
        self.set_transaction_serializable(&mut tx).await?;

        // 条件：
        // - 指定の蔵書 ID をもつ蔵書が存在する
        // - その蔵書が貸出中である
        // - その蔵書を借りたユーザーが指定のユーザーと同じである
        //
        // ちなみにブロックはres 変数がシャドーイングで上書きされるのを防ぐため使用
        {
            let res = sqlx::query_as!(
                CheckoutStateRow,
                r#"
                    SELECT
                    b.book_id,
                    c.checkout_id AS "checkout_id?: CheckoutId",
                    c.user_id AS "user_id?: UserId"
                    FROM books AS b
                    LEFT OUTER JOIN checkouts AS c USING(book_id)
                    WHERE book_id = $1;
                "#,
                event.book_id as _,
            )
            .fetch_optional(&mut *tx)
            .await
            .map_err(AppError::SpecificOperationError)?;

            match res {
                // 条件を満たさなければエラーを返す
                None => {
                    return Err(AppError::EntityNotFound(format!(
                        " 書籍（{}）が見つかりませんでした。",
                        event.book_id
                    )))
                }
                Some(CheckoutStateRow {
                    checkout_id: Some(c),
                    user_id: Some(u),
                    ..
                }) if (c, u) != (event.checkout_id, event.returned_by) => {
                    return Err(AppError::UnprocessableEntity(format!(
                        " 指定の貸出（ID（{}）, ユーザー（{}）, 書籍（{}））は返却できません。",
                        event.checkout_id, event.returned_by, event.book_id
                    )))
                }
                // それ以外は処理続行
                _ => {}
            }
        }

        // checkoutsテーブルにある該当貸出レコードにreturned_atを追加してreturned_checkoutsテーブルにINSERT
        let res = sqlx::query!(
            r#"
                INSERT INTO returned_checkouts
                (checkout_id, book_id, user_id, checked_out_at, returned_at)
                SELECT checkout_id, book_id, user_id, checked_out_at, $2
                FROM checkouts
                WHERE checkout_id = $1
                ;
            "#,
            event.checkout_id as _,
            event.returned_at,
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::SpecificOperationError)?;

        if res.rows_affected() < 1 {
            return Err(AppError::NoRowsAffectedError(
                "No returning record has been updated".into(),
            ));
        }

        // checkoutsテーブルから該当貸出IDのレコードを削除
        let res = sqlx::query!(
            r#"
                DELETE FROM checkouts WHERE checkout_id = $1;
            "#,
            event.checkout_id as _,
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::SpecificOperationError)?;

        if res.rows_affected() < 1 {
            return Err(AppError::NoRowsAffectedError(
                "No checkout record has been deleted".into(),
            ));
        }

        tx.commit().await.map_err(AppError::TransactionError)?;

        Ok(())
    }

    // すべての貸出中の情報を取得
    async fn find_unreturned_all(&self) -> AppResult<Vec<Checkout>> {
        // checkoutsをbooksとINNER JOINしつつ全件抽出
        sqlx::query_as!(
            CheckoutRow,
            r#"
                SELECT
                c.checkout_id,
                c.book_id,
                c.user_id,
                c.checked_out_at,
                b.title,
                b.author,
                b.isbn
                FROM checkouts AS c
                INNER JOIN books AS b USING(book_id)
                ORDER BY c.checked_out_at ASC
                ;
            "#,
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map(|rows| rows.into_iter().map(Checkout::from).collect())
        .map_err(AppError::SpecificOperationError)
    }

    // 特定のユーザーの貸出中の情報を取得
    async fn find_unreturned_by_user_id(&self, user_id: UserId) -> AppResult<Vec<Checkout>> {
        // find_unreturned_allのSQL文に`WHERE c.user_id = $1`を追加しただけ
        sqlx::query_as!(
            CheckoutRow,
            r#"
                SELECT
                c.checkout_id,
                c.book_id,
                c.user_id,
                c.checked_out_at,
                b.title,
                b.author,
                b.isbn
                FROM checkouts AS c
                INNER JOIN books AS b USING(book_id)
                WHERE c.user_id = $1
                ORDER BY c.checked_out_at ASC
                ;
            "#,
            user_id as _
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map(|rows| rows.into_iter().map(Checkout::from).collect())
        .map_err(AppError::SpecificOperationError)
    }

    // 特定の蔵書の貸出履歴を取得
    // このメソッドではreturned/unreturnedの両方をまとめて返す必要がある
    async fn find_history_by_book_id(&self, book_id: BookId) -> AppResult<Vec<Checkout>> {
        // 未返却の貸出情報を取得
        let checkout: Option<Checkout> = self.find_unreturned_by_book_id(book_id).await?;
        // 返却済みの貸出情報を取得
        let mut checkout_histories: Vec<Checkout> = sqlx::query_as!(
            ReturnedCheckoutRow,
            r#"
                SELECT
                rc.checkout_id,
                rc.book_id,
                rc.user_id,
                rc.checked_out_at,
                rc.returned_at,
                b.title,
                b.author,
                b.isbn
                FROM returned_checkouts AS rc
                INNER JOIN books AS b USING(book_id)
                WHERE rc.book_id = $1
                ORDER BY rc.checked_out_at DESC
            "#,
            book_id as _
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?
        .into_iter()
        .map(Checkout::from)
        .collect();

        // 貸出中の項目が存在する場合は返却済みの履歴の先頭に追加する
        if let Some(co) = checkout {
            checkout_histories.insert(0, co);
        }

        Ok(checkout_histories)
    }
}

impl CheckoutRepositoryImpl {
    // トランザクション分離レベルをSERIALIZABLEにするために内部的に使うメソッド
    async fn set_transaction_serializable(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> AppResult<()> {
        sqlx::query!("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE")
            .execute(&mut **tx)
            .await
            .map_err(AppError::SpecificOperationError)?;
        Ok(())
    }

    // 未返却の貸出情報を取得するために内部的に使うメソッド
    async fn find_unreturned_by_book_id(&self, book_id: BookId) -> AppResult<Option<Checkout>> {
        let res = sqlx::query_as!(
            CheckoutRow,
            r#"
                SELECT
                c.checkout_id,
                c.book_id,
                c.user_id,
                c.checked_out_at,
                b.title,
                b.author,
                b.isbn
                FROM checkouts AS c
                INNER JOIN books AS b USING(book_id)
                WHERE c.book_id = $1
            "#,
            book_id as _,
        )
        .fetch_optional(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?
        .map(Checkout::from);

        Ok(res)
    }
}
