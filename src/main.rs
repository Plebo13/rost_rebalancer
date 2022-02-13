use model::asset::Asset;
use postgres::{Client, NoTls};
use std::collections::HashMap;

use crate::model::asset_class::AssetClass;
mod model;

fn main() {
    let mut client = Client::connect(
        "postgresql://postgres@127.0.0.1:5432/rost_rebalancer",
        NoTls,
    )
    .unwrap();

    let assets = get_assets(&mut client);
    let mut asset_classes = get_asset_classes(&mut client, &assets);
    for asset_class in &mut asset_classes {
        asset_class.resolve(&mut client, &assets);
        asset_class.print();
    }

    let mut input = String::new();
    println!("How much money should be invested?");
    let _b1 = std::io::stdin().read_line(&mut input).unwrap();
    //println!("---{}---",input);
    let investment_amount = input.trim().parse::<f32>().unwrap();
    for asset_class in &mut asset_classes {
        asset_class.invest(investment_amount);
    }
}

fn get_asset_classes(client: &mut Client, assets: &HashMap<String, Asset>) -> Vec<AssetClass> {
    let mut asset_classes: Vec<AssetClass> = Vec::new();
    for row in client
        .query("SELECT id, name FROM asset_classes", &[])
        .unwrap()
    {
        let mut asset_class = AssetClass {
            id: row.get(0),
            name: row.get(1),
            value: 0.0,
            classifications: Vec::new(),
        };

        let query = format!("SELECT id FROM assets WHERE asset_class={}", asset_class.id);
        for assets_row in client.query(&query, &[]).unwrap() {
            let asset_id: String = assets_row.get(0);
            asset_class.value += assets.get(&asset_id).unwrap().value;
        }
        asset_classes.push(asset_class);
    }
    return asset_classes;
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
