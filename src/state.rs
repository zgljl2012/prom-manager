use std::{collections::HashMap, fs, path::Path, error::Error, sync::{Arc, Mutex}};

use actix_web::{web, error};
use log::warn;
use serde::{Serialize, Deserialize};
use futures::StreamExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Machine {
    targets: Vec<String>,
    labels: HashMap<String, String>,
}

const MAX_SIZE: usize = 262_144; // max payload size is 256k

impl Machine {
    pub fn new(targets: Vec<String>, labels: HashMap<String, String>) -> Self {
        Self {
            targets,
            labels
        }
    }
    pub async fn from_payload(mut payload: web::Payload) -> Result<Self, Box<dyn Error>> {
        let mut body = web::BytesMut::new();
        while let Some(chunk) = payload.next().await {
            let chunk = chunk?;
            // limit max size of in-memory payload
            if (body.len() + chunk.len()) > MAX_SIZE {
                return Err(Box::new(error::ErrorBadRequest("overflow")));
            }
            body.extend_from_slice(&chunk);
        }
        // body is loaded, now we can deserialize serde-json
        let obj = serde_json::from_slice::<Machine>(&body)?;
        Ok(obj)
    }
}

#[derive(Debug, Clone)]
pub struct MachineManager {
    cfg_path: String,
    machines: Vec<Machine>
}

fn load(cfg_path: String) -> Vec<Machine> {
    let mut contents = "[]".to_string();
    if !Path::new(&cfg_path).exists() {
        warn!("{} does not exist", cfg_path)
    } else {
        contents = fs::read_to_string(&cfg_path)
        .expect("Should have been able to read the file");
    }
    let machines = serde_json::from_str::<Vec<Machine>>(&contents);
    match machines {
        Ok(machines) => {
            machines
        },
        Err(err) => panic!("Load failed: {:?}", err)
    }
}

impl MachineManager {
    pub fn new(cfg_path: String) -> Self {
        let machines = load(cfg_path.clone());
        Self {
            cfg_path,
            machines,
        }
    }
    fn save(&self) {
        fs::write(&self.cfg_path, self.to_json()).expect("Should have been able to write the file");
    }
    pub fn add_machine(&mut self, machine: Machine) {
        self.machines.push(machine);
        self.save();
    }
    pub fn remove_machine(&mut self, id: usize) {
        self.machines.remove(id);
        self.save();
    }
    pub fn get_machine(&mut self, id: usize) -> Option<&Machine> {
        let m = self.machines.get(id);
        return m.clone()
    }
    pub fn get_matches(&mut self) -> Vec<Machine> {
        self.machines.clone()
    }
    pub fn size(&self) -> usize {
        self.machines.len()
    }
    pub fn to_vec(&self) -> Vec<Machine> {
        self.machines.clone()
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.to_vec()).expect("Should have been able to serialize the hashmap")
    }
}

pub struct AppState {
    pub wechat_robot: Option<String>,
    pub machine_manager: Arc<Mutex<MachineManager>>,
    pub service_manager: Arc<Mutex<MachineManager>>,
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::state::MachineManager;
    use super::Machine;

    #[test]
    fn test_machines() {
        let mut m = MachineManager::new("./config_machines.json".to_string());
        let mut labels = HashMap::new();
        labels.insert("name".to_string(), "test".to_string());
        m.add_machine(Machine::new(vec!("target".to_string()), labels));
        assert_eq!(m.size(), 1);
        let got = m.get_machine(0);
        assert_eq!(got.is_some(), true);
        m.remove_machine(2);
        assert_eq!(m.size(), 1);
        m.remove_machine(0);
        assert_eq!(m.size(), 0);
    }
}
