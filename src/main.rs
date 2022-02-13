use postgres::{Client, NoTls};

use crate::model::asset_class::AssetClass;
mod model;

fn main() {
    let mut client = Client::connect(
        "postgresql://postgres@127.0.0.1:5432/rost_rebalancer",
        NoTls,
    )
    .unwrap();

    update_assets(&mut client);
    let mut asset_classes = get_asset_classes(&mut client);
    for asset_class in &mut asset_classes {
        asset_class.resolve(&mut client);
        asset_class.print();
    }

    let mut input = String::new();
    println!("How much money should be invested?");
    let _b1 = std::io::stdin().read_line(&mut input).unwrap();
    //println!("---{}---",input);
    let investment_amount = input.trim().parse::<f32>().unwrap();
    for asset_class in &mut asset_classes {
        asset_class.invest(investment_amount, &mut client);
    }
}

fn get_asset_classes(client: &mut Client) -> Vec<AssetClass> {
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

        let query = format!(
            "SELECT value FROM assets WHERE asset_class={}",
            asset_class.id
        );
        for assets_row in client.query(&query, &[]).unwrap() {
            let asset_value: f32 = assets_row.get(0);
            asset_class.value += asset_value;
        }
        asset_classes.push(asset_class);
    }
    return asset_classes;
}

fn update_assets(client: &mut Client) {
    for asset_row in client
        .query("SELECT id, quantity FROM assets", &[])
        .unwrap()
    {
        let id: String = asset_row.get(0);
        let quantity: f32 = asset_row.get(1);
        let value: f32 = rost_app::get_etf_price(id.clone()).unwrap() * quantity;

        let update_query = format!(
            "UPDATE assets
            SET value={:.2} 
            WHERE id='{}'",
            value, id
        );
        client.query(&update_query, &[]).unwrap();
    }
}
