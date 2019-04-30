/// Functions for generating test data
pub mod generate {
    use crate::models::{NewArticle, NewUser};
    use fake::fake;

    pub fn new_user() -> NewUser {
        NewUser {
            username: fake!(Internet.user_name).to_string(),
            email: fake!(Internet.free_email).to_string(),
            password: fake!(Lorem.word).to_string(),
        }
    }

    pub fn new_article(user_id: i32) -> NewArticle {
        NewArticle {
            title: fake!(Lorem.sentence(4, 10)).to_string(),
            slug: format!("{}{}", fake!(Lorem.word).to_string(), user_id),
            description: fake!(Lorem.paragraph(3, 10)),
            body: fake!(Lorem.paragraph(10, 5)),
            user_id: user_id,
        }
    }
}
