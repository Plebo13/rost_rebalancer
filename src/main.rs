use postgres::{Client, NoTls};
fn main() {
    let mut client = Client::connect(
        "postgresql://postgres@127.0.0.1:5432/rost_rebalancer",
        NoTls,
    )
    .unwrap();

    for row in client
        .query("SELECT id, name, quantity FROM assets", &[])
        .unwrap()
    {
        let id: &str = row.get(0);
        let name: &str = row.get(1);
        let quantity: f32 = row.get(2);

        println!("Found asset: {} {} {:?}", id, name, quantity);
    }
}
