use postgres::{Client, NoTls};
use std::collections::HashMap;
mod model;
use crate::model::*;

fn main() {
    let mut client = Client::connect(
        "postgresql://postgres@127.0.0.1:5432/rost_rebalancer",
        NoTls,
    )
    .unwrap();

    let assets = get_assets(&mut client);
    get_asset_classes(&mut client, &assets);
}

fn get_asset_classes(client: &mut Client, assets: &HashMap<String, Asset>) {
    for asset_class_row in client
        .query("SELECT id, name FROM asset_classes", &[])
        .unwrap()
    {
        let id: i32 = asset_class_row.get(0);
        let name: String = asset_class_row.get(1);
        let value: f32 = 0.0;
        let mut asset_class = AssetClass { id, name, value };

        let query = format!("SELECT id FROM assets WHERE asset_class={}", id);
        for assets_row in client.query(&query, &[]).unwrap() {
            let asset_id: String = assets_row.get(0);
            asset_class.value += assets.get(&asset_id).unwrap().value;
        }

        print!("{:<23}{:.2}€\n", asset_class.name, asset_class.value);

        get_classifications(client, &asset_class, assets);
    }
}

fn get_classifications(
    client: &mut Client,
    asset_class: &AssetClass,
    assets: &HashMap<String, Asset>,
) {
    let query = format!(
        "SELECT id, name FROM classifications WHERE asset_class={}",
        asset_class.id
    );
    for classifications_row in client.query(&query, &[]).unwrap() {
        let id: i32 = classifications_row.get(0);
        let classification: String = classifications_row.get(1);
        let mut classification_value: f32 = 0.0;
        let mut classes: Vec<Class> = Vec::new();
        print!("  {}\n", classification);

        let query = format!(
            "SELECT id, name, allocation FROM classes WHERE classification={}",
            id
        );
        for class_row in client.query(&query, &[]).unwrap() {
            let class_id: i32 = class_row.get(0);
            let class_name: String = class_row.get(1);
            let class_allocation: f32 = class_row.get(2);
            let mut class_value: f32 = 0.0;

            let query = format!("SELECT asset FROM asset_mapping WHERE class={}", class_id);
            for asset_row in client.query(&query, &[]).unwrap() {
                let asset_id: String = asset_row.get(0);
                classification_value += assets.get(&asset_id).unwrap().value;
                class_value += assets.get(&asset_id).unwrap().value;
            }
            let class = Class {
                id: class_id,
                name: class_name,
                allocation: class_allocation,
                value: class_value,
            };
            classes.push(class);
        }

        // Print the classes for this classification.
        for class in classes {
            print!(
                "    {:<18} {:>8.2}€ {:.2}% ({:.2}%)\n",
                class.name,
                class.value,
                class.allocation,
                class.value / classification_value * 100.0
            );
        }
    }
}

fn get_assets(client: &mut Client) -> HashMap<String, Asset> {
    let mut assets: HashMap<String, Asset> = HashMap::new();

    for asset_row in client
        .query("SELECT id, name, enabled, quantity FROM assets", &[])
        .unwrap()
    {
        let id: String = asset_row.get(0);
        let name: String = asset_row.get(1);
        let enabled: bool = asset_row.get(2);
        let quantity: f32 = asset_row.get(3);
        let value: f32 = rost_app::get_etf_price(id.clone()).unwrap() * quantity;
        let asset = Asset {
            name,
            enabled,
            quantity,
            value,
        };

        assets.insert(id, asset);
    }

    return assets;
}
