pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    // to run this test, you have to set .env file with DATABASE_URL
    // DATABASE_URL=postgresql://localhost:5432/sqlx_test_example?user=my_user&password=my_password

    #[sqlx::test]
    async fn it_works(pool: sqlx::PgPool) {
        // check connection
        let row = sqlx::query!("SELECT 1 + 1 AS result")
            .fetch_one(&pool)
            .await
            .unwrap();
        let result = row.result;
        assert_eq!(result, Some(2));
    }

    #[sqlx::test(fixtures("common"))]
    async fn test_with_fixture(pool: sqlx::PgPool) {
        // check connection
        let row = sqlx::query!("SELECT author FROM books WHERE title = 'Test book 1'")
            .fetch_one(&pool)
            .await
            .unwrap();
        let result = row.author;
        assert_eq!(result, "Test author 1".to_string());
    }
}
