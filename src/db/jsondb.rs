use std::{fs, path::PathBuf};
use chrono::{Duration, Utc};
use crate::db::{Database, Client, Payment, Target};

pub struct JsonDb {
    file_path: PathBuf,
    clients: Option<Vec<Client>>
}

impl JsonDb {
    pub fn new(file_path: PathBuf) -> Result<JsonDb, String> {
        if !file_path.is_file() {
            fs::write(&file_path, "[]").map_err(|e| format!("cannot create database file at '{}': {}",
                file_path.to_str().unwrap(), e.to_string()))?;
        }

        Ok(JsonDb {
            file_path,
            clients: None
        })
    }
}

impl Database for JsonDb {
    fn add_client(&mut self, name: &str, days: u32, seller: &str, money: u32, info: &str) -> Result<(), String> {
        let mut clients: Vec<Client> = self.list_clients()?;
        if clients.iter().any(|exist_client| { exist_client.name == name }) {
            return Err(format!("client '{}' already exists!", name))
        }

        let client = Client::new(name, days, &seller, money, info);
        clients.push(client);
        self.clients = Some(clients);
        Ok(())
    }

    fn renew_client(&mut self, name: &str, days: u32, seller: &str, money: u32) -> Result<(), String> {
        let mut clients: Vec<Client> = self.list_clients()?;
        let index = match clients.iter().position(|client| { client.name == name }) {
            Some(index) => index,
            None => return Err(format!("client with name '{}' doesn't exists!", name))
        };

        let mut client = &mut clients[index];
        let now_date = Utc::now();

        if now_date > client.expire_time {
            client.expire_time = now_date;
        }

        client.expire_time += Duration::days(days.into());
        client.payments.push(Payment { seller: seller.to_string(), money, date: now_date });
        self.clients = Some(clients);
        Ok(())
    }

    fn renew_all_clients(&mut self, days: u32) -> Result<(), String> {
        let mut clients = self.list_clients()?;
        let now_date = Utc::now();

        for client in clients.iter_mut() {
            let is_expired = client.expire_time < now_date; 
            if is_expired {
                continue;
            }

            client.expire_time += Duration::days(days.into());
        }

        self.clients = Some(clients);
        Ok(())
    }

    fn remove_client(&mut self, name: &str) -> Result<(), String> {
        let mut clients: Vec<Client> = self.list_clients()?;
        let index: usize = match clients.iter().position(|client| { client.name == name }) {
            Some(index) => index,
            None => return Err(format!("client with name '{}' doesn't exists!", name))
        };

        clients.remove(index);
        self.clients = Some(clients);
        Ok(())
    }

    fn list_clients(&self) -> Result<Vec<Client>, String> {
        if let Some(clients) = &self.clients {
            return Ok(clients.clone());
        }

        let file = fs::File::open(&self.file_path).map_err(
            |error| format!("cannot open file '{}': {}", self.file_path.to_str().unwrap(), error.to_string()))?;
        serde_json::from_reader(file).map_err(|error| format!("cannot parse json: {}", error.to_string()))
    }

    fn rename_client(&mut self, old_name: &str, new_name: &str) -> Result<(), String> {
        let mut clients: Vec<Client> = self.list_clients()?;
        let mut client = clients.iter_mut().find(|client| client.name == old_name)
            .ok_or(format!("client with name '{}' doesn't exists!", old_name))?;
        client.name = new_name.to_string();

        self.clients = Some(clients);
        Ok(())
    }

    fn set_client_info(&mut self, target: Target, info: &str) -> Result<(), String> {
        let mut clients: Vec<Client> = self.list_clients()?;

        for client in clients.iter_mut() {
            if let Target::OnePerson(name) = &target {
                if name != &client.name {
                    continue;
                }
            }

            client.info = Some(info.to_string());
        }

        self.clients = Some(clients);
        Ok(())
    }

    fn get_client_info(&self, name: &str) -> Result<String, String> {
        let clients: Vec<Client> = self.list_clients()?;
        if let Some(client) = clients.into_iter().find(|client| client.name == name) {
            let info = client.info.unwrap_or("".to_string());
            return Ok(info.to_string());
        }

        Err(format!("cannot find client with name '{}'", name))
    }

    fn commit(self) -> Result<(), String> {
        if let Some(clients) = self.clients {
            let json_string = serde_json::to_string_pretty(&clients).unwrap();
            fs::write(&self.file_path, json_string.as_bytes()).map_err(
                |error| format!("cannot write to file '{}': {}", self.file_path.to_str().unwrap(), error.to_string()))?;
        }

        Ok(())
    }
}
