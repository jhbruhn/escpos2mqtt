#[tokio::main]
async fn main() -> Result<(), ()> {
    let printers = escpos2mqtt::printer::discover_network().await;

    println!("{:?}", printers);

    Ok(())
}
