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
    let investment = input.trim().parse::<f32>().unwrap();
    for asset_class in &mut asset_classes {
        asset_class.invest(investment, &mut client);
    }

    calculate_assets_delta(&mut client);
    invest(&mut client, investment);
    print_result(&mut client);
}

fn print_result(client: &mut Client) {
    println!("Invest:");
    for asset_row in client
        .query(
            "SELECT name, investment FROM assets WHERE investment>0 ORDER BY investment",
            &[],
        )
        .unwrap()
    {
        let name: String = asset_row.get(0);
        let investment: f32 = asset_row.get(1);
        println!("{}: {:.2}", name, investment);
    }
}

fn invest(client: &mut Client, mut investment: f32) {
    for asset_row in client
        .query("SELECT id, delta FROM assets ORDER BY ter, value", &[])
        .unwrap()
    {
        if investment > 0.0 {
            let mut invested: bool = false;
            let asset_id: String = asset_row.get(0);
            let mut asset_delta: f32 = asset_row.get(1);
            if investment < asset_delta {
                asset_delta = investment;
            }

            let query = format!(
                "SELECT id, delta
                FROM classes
                WHERE EXISTS (SELECT id
                    FROM asset_mapping
                    WHERE class=classes.id AND asset='{}')
                ORDER BY delta",
                asset_id
            );

            for class_row in client.query(&query, &[]).unwrap() {
                let class_id: i32 = class_row.get(0);
                let mut class_delta: f32 = class_row.get(1);

                if class_delta <= 0.0 {
                    break;
                } else if class_delta >= asset_delta {
                    class_delta -= asset_delta;
                } else {
                    asset_delta = class_delta;
                    class_delta = 0.0;
                }

                let update_query = format!(
                    "UPDATE classes
                    SET delta={:.2} 
                    WHERE id='{}'",
                    class_delta, class_id
                );
                client.query(&update_query, &[]).unwrap();
                invested = true;
            }

            if invested {
                let update_query = format!(
                    "UPDATE assets
                    SET investment={:.2} 
                    WHERE id='{}'",
                    asset_delta, asset_id
                );
                client.query(&update_query, &[]).unwrap();

                investment -= asset_delta;
            }
        }
    }
}

fn calculate_assets_delta(client: &mut Client) {
    for asset_row in client.query("SELECT id FROM assets", &[]).unwrap() {
        let asset_id: String = asset_row.get(0);
        let mut delta: f32 = 0.0;

        let query = format!(
            "SELECT delta
            FROM classes
            WHERE EXISTS (SELECT id
                FROM asset_mapping
                WHERE class=classes.id AND asset='{}')",
            asset_id
        );
        for class_row in client.query(&query, &[]).unwrap() {
            let class_delta: f32 = class_row.get(0);
            if 0.0 < class_delta {
                if delta == 0.0 || class_delta < delta {
                    delta = class_delta;
                }
            }
        }

        let update_query = format!(
            "UPDATE assets
            SET delta={:.2} 
            WHERE id='{}'",
            delta, asset_id
        );
        client.query(&update_query, &[]).unwrap();
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
        println!("Updating {}", id);
        let quantity: f32 = asset_row.get(1);
        let value: f32 = rost_app::get_etf_price(id.clone()).unwrap() * quantity;

        let update_query = format!(
            "UPDATE assets
            SET investment=0, delta=0, value={:.2} 
            WHERE id='{}'",
            value, id
        );
        client.query(&update_query, &[]).unwrap();
    }

    // Set all deltas to '0' in the classes table.
    client.query("UPDATE classes SET delta=0", &[]).unwrap();
}
