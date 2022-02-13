use std::collections::HashMap;

use postgres::Client;

use super::{asset::Asset, classification::Classification};

pub struct Class {
    pub id: i32,
    pub name: String,
    pub allocation: f32,
    pub value: f32,
    pub investment: f32,
    pub parent_value: f32,
    pub classifications: Vec<Classification>,
}

impl Class {
    pub fn resolve(&mut self, client: &mut Client, assets: &HashMap<String, Asset>) {
        let mut query = format!(
            "SELECT asset
            FROM asset_mapping
            WHERE class={}",
            self.id
        );
        for row in client.query(&query, &[]).unwrap() {
            let asset_id: String = row.get(0);
            self.value += assets.get(&asset_id).unwrap().value;
        }

        query = format!(
            "SELECT id, name
            FROM classifications 
            WHERE EXISTS (SELECT parent
                FROM classification_mapping
                WHERE child=classifications.id
                AND parent={})",
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

    pub fn invest(&mut self, investment: f32, client: &mut Client) {
        let goal_value: f32 = (self.parent_value + investment) * self.allocation / 100.0;
        let mut delta = goal_value - self.value;
        if delta > investment {
            delta = investment;
        }

        if self.classifications.len() > 0 {
            for classification in &mut self.classifications {
                classification.invest(delta, client);
            }
        } else {
            let query = format!(
                "UPDATE classes
                SET delta={:.2} 
                WHERE id={}",
                delta, self.id
            );
            client.query(&query, &[]).unwrap();
        }
    }

    pub fn print(&mut self, indent: &String) {
        let percentage: f32 = self.value / self.parent_value * 100.0;
        if self.classifications.len() > 0 {
            println!("{}{} - ({:.2}%)", indent, self.name, percentage);
            for classification in &mut self.classifications {
                classification.print(&format!("  {}", indent));
            }
        } else {
            println!(
                "{}{} - {:.2}€ ({:.2}%)",
                indent, self.name, self.value, percentage
            );
        }
    }
}
