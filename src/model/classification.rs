use std::collections::HashMap;

use postgres::Client;

use super::{asset::Asset, class::Class};

pub struct Classification {
    pub id: i32,
    pub name: String,
    pub value: f32,
    pub classes: Vec<Class>,
}

impl Classification {
    pub fn resolve(&mut self, client: &mut Client, assets: &HashMap<String, Asset>) {
        let query = format!(
            "SELECT id, name, allocation
            FROM classes
            WHERE classification={}",
            self.id
        );
        for class_row in client.query(&query, &[]).unwrap() {
            let mut class = Class {
                id: class_row.get(0),
                name: class_row.get(1),
                allocation: class_row.get(2),
                value: 0.0,
                investment: 0.0,
                parent_value: self.value,
                classifications: Vec::new(),
            };
            class.resolve(client, assets);
            self.classes.push(class);
        }
    }

    pub fn invest(&mut self, investment: f32) {
        for class in &mut self.classes {
            class.invest(investment);
        }
    }

    pub fn print(&mut self, indent: &String) {
        println!("{}{}", indent, self.name);
        for class in &mut self.classes {
            class.print(&format!("  {}", indent));
        }
    }
}
