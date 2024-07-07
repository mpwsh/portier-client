use portier_client::{Client, Result};
use std::io::{stdin, stdout, Write};

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = Client::builder()
        .with_store("custom_cookies.json")
        .with_rpc_addr("http://127.0.0.1:8000")
        .with_broker_addr("http://127.0.0.1:3333")
        .with_session_cookie_domain("127.0.0.1")
        .build()?;

    if client.session().is_none() {
        println!(
            "[!] Unable to find valid session, please login (You'll receive an email with a code to input next):"
        );
        let email = {
            print!("[.] Email: ");
            stdout().flush()?;
            let mut email = String::new();
            stdin().read_line(&mut email)?;
            email.trim().to_owned()
        };

        client.login(&email).await?;
        println!("Initializing session");

        let code = {
            print!("[.] Authorization code: ");
            stdout().flush()?;
            let mut code = String::new();
            stdin().read_line(&mut code)?;
            code.trim().to_owned()
        };

        client.confirm(&code).await?;
        client.save_session().await?;

        println!("[~] Session initialized and saved");
    } else {
        println!("[~] Found active session");
    }

    match client.whoami().await {
        Ok(user_data) => {
            if let Some(email) = user_data.email {
                println!("[~] Logged in as: {}", email);
            } else {
                println!("[!] Logged in, but unable to retrieve email");
            }
        }
        Err(_) => {
            println!("[!] Unable to retrieve user data. Please try logging in again.");
        }
    }

    Ok(())
}
