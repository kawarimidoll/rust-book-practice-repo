use kernel::model::book::{event::CreateBook, Book};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBookRequest {
    pub title: String,
    pub author: String,
    pub isbn: String,
    pub description: String,
}
impl From<CreateBookRequest> for CreateBook {
    fn from(req: CreateBookRequest) -> Self {
        let CreateBookRequest {
            title,
            author,
            isbn,
            description,
        } = req;
        Self {
            title,
            author,
            isbn,
            description,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookResponse {
    pub id: Uuid,
    pub title: String,
    pub author: String,
    pub isbn: String,
    pub description: String,
}
impl From<Book> for BookResponse {
    fn from(book: Book) -> Self {
        let Book {
            id,
            title,
            author,
            isbn,
            description,
            ..
        } = book;
        Self {
            id,
            title,
            author,
            isbn,
            description,
        }
    }
}
