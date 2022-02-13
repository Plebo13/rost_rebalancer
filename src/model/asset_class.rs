use std::collections::HashMap;

use postgres::Client;

use super::{asset::Asset, classification::Classification};

pub struct AssetClass {
    pub id: i32,
    pub name: String,
    pub value: f32,
    pub classifications: Vec<Classification>,
}

impl AssetClass {
    pub fn resolve(&mut self, client: &mut Client, assets: &HashMap<String, Asset>) {
        let query = format!(
            "SELECT id, name
            FROM classifications
            WHERE asset_class={}
            AND NOT EXISTS (SELECT parent
                FROM classification_mapping
                WHERE child=classifications.id)",
            self.id
        );
        for row in client.query(&query, &[]).unwrap() {
            let mut classification = Classification {
                id: row.get(0),
                name: row.get(1),
                value: self.value,
                classes: Vec::new(),
            };
            classification.resolve(client, assets);
            self.classifications.push(classification);
        }
    }

    pub fn invest(&mut self, investment: f32) {
        for classification in &mut self.classifications {
            classification.invest(investment);
        }
    }

    pub fn print(&mut self) {
        println!("{} - {:.2}€", self.name, self.value);
        for classification in &mut self.classifications {
            classification.print(&String::from("  "));
        }
    }
}
