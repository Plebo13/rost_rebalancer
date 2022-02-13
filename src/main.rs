use postgres::{Client, NoTls};
use std::collections::HashMap;

struct Asset {
    name: String,
    quantity: f32,
    value: f32,
}

fn main() {
    let mut client = Client::connect(
        "postgresql://postgres@127.0.0.1:5432/rost_rebalancer",
        NoTls,
    )
    .unwrap();

    let mut assets = get_assets(&mut client);
    get_asset_classes(&mut client, &assets);
}

fn get_asset_classes(client: &mut Client, assets: &HashMap<String, Asset>) {
    for asset_class_row in client
        .query("SELECT id, name FROM asset_classes", &[])
        .unwrap()
    {
        let id: i32 = asset_class_row.get(0);
        let asset_class: String = asset_class_row.get(1);
        let mut value: f32 = 0.0;

        let query = format!("SELECT id FROM assets WHERE asset_class={}", id);
        for assets_row in client.query(&query, &[]).unwrap() {
            let asset_id: String = assets_row.get(0);
            value += assets.get(&asset_id).unwrap().value;
        }

        print!("{:<8}{:.2}€\n", asset_class, value);

        let query = format!(
            "SELECT id, name FROM classifications WHERE asset_class={}",
            id
        );
        for classifications_row in client.query(&query, &[]).unwrap() {
            let classification: String = classifications_row.get(1);
            print!("  {:<15}0.00€\n", classification);
        }
    }
}

fn get_assets(client: &mut Client) -> HashMap<String, Asset> {
    let mut assets: HashMap<String, Asset> = HashMap::new();

    for asset_row in client
        .query(
            "SELECT id, name, quantity FROM assets WHERE enabled=true",
            &[],
        )
        .unwrap()
    {
        let id: String = asset_row.get(0);
        let name: String = asset_row.get(1);
        let quantity: f32 = asset_row.get(2);
        let value: f32 = 0.0; // TODO: Use rost_app to calculate value.
        let asset = Asset {
            name,
            quantity,
            value,
        };

        assets.insert(id, asset);
    }

    return assets;
}
