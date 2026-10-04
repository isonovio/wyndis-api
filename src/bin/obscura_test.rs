use std::env;

use anyhow::{Context, Result};
use obscura::Browser;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let url = env::args().nth(1).context("Missing URL")?;
    let browser = Browser::builder().stealth(true).build()?;
    let mut page = browser.new_page().await?;
    page.goto(&url).await?;
    page.settle(3000).await;
    println!("{}", page.content());
    Ok(())
}
