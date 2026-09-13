pub mod card;
pub mod error;
pub mod info;
pub mod login;
pub mod personal;
pub mod task;
pub mod term;

mod util;

#[cfg(test)]
mod test;

#[cfg(test)]
mod tests {
    use crate::{iportal::login::get_iportal_token, test::TestResult};

    #[tokio::test]
    #[ignore]
    async fn test_login() -> TestResult<()> {
        let token = get_iportal_token().await?;
        println!("{token:#?}");
        Ok(())
    }
}
