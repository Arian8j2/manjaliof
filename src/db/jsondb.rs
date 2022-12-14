use std::fs;
use chrono::{Duration, Utc};
use crate::db::{Database, BackupableDatabase, Client, Payment, Target};

pub struct JsonDb {
    file_path: String
}

impl JsonDb {
    pub fn new(file_path: &str) -> JsonDb {
        JsonDb {
            file_path: file_path.to_string()
        }
    }

    fn save_clients(&self, clients: Vec<Client>) -> Result<(), String> {
        let json_string = serde_json::to_string_pretty(&clients).unwrap();
        fs::write(&self.file_path, json_string.as_bytes()).map_err(
            |error| format!("cannot write to file '{}': {}", self.file_path, error.to_string()))
    }
}

impl Database for JsonDb {
    fn add_client(&self, name: &str, days: u32, seller: &str, money: u32, info: &str) -> Result<(), String> {
        let mut clients: Vec<Client> = self.list_clients()?;
        if clients.iter().any(|exist_client| { exist_client.name == name }) {
            return Err(format!("client '{}' already exists!", name))
        }

        let client = Client::new(name, days, &seller, money, info);
        clients.push(client);
        self.save_clients(clients)?;
        Ok(())
    }

    fn renew_client(&self, name: &str, days: u32, seller: &str, money: u32) -> Result<(), String> {
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
        self.save_clients(clients)?;
        Ok(())
    }

    fn renew_all_clients(&self, days: u32) -> Result<(), String> {
        let mut clients = self.list_clients()?;
        let now_date = Utc::now();

        for client in clients.iter_mut() {
            let is_expired = client.expire_time < now_date; 
            if is_expired {
                continue;
            }

            client.expire_time += Duration::days(days.into());
        }

        self.save_clients(clients)?;
        Ok(())
    }

    fn remove_client(&self, name: &str) -> Result<(), String> {
        let mut clients: Vec<Client> = self.list_clients()?;
        let index: usize = match clients.iter().position(|client| { client.name == name }) {
            Some(index) => index,
            None => return Err(format!("client with name '{}' doesn't exists!", name))
        };

        clients.remove(index);
        self.save_clients(clients)?;
        Ok(())
    }

    fn list_clients(&self) -> Result<Vec<Client>, String> {
        let file = fs::File::open(&self.file_path).map_err(
            |error| format!("cannot open file '{}': {}", self.file_path, error.to_string()))?;

        serde_json::from_reader(file).map_err(|error| format!("cannot parse json: {}", error.to_string()))
    }

    fn rename_client(&self, old_name: &str, new_name: &str) -> Result<(), String> {
        let mut clients: Vec<Client> = self.list_clients()?;
        let mut client = clients.iter_mut().find(|client| client.name == old_name)
            .ok_or(format!("client with name '{}' doesn't exists!", old_name))?;
        client.name = new_name.to_string();

        self.save_clients(clients)?;
        Ok(())
    }

    fn set_client_info(&self, target: Target, info: &str) -> Result<(), String> {
        let mut clients: Vec<Client> = self.list_clients()?;

        for client in clients.iter_mut() {
            if let Target::OnePerson(name) = &target {
                if name != &client.name {
                    continue;
                }
            }

            client.info = Some(info.to_string());
        }

        self.save_clients(clients)?;
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
}

impl BackupableDatabase for JsonDb {
    type DbData = String; 

    fn get_backup(&self) -> Result<Self::DbData, String> {
        let file_content = fs::read_to_string(&self.file_path).map_err(
            |error| format!("cannot open database file at '{}' for getting backup: {}",
                self.file_path, error.to_string()))?;

        Ok(file_content)
    }

    fn restore_backup(&self, backup: Self::DbData) -> Result<(), String> {
        fs::write(&self.file_path, backup).map_err(
            |error| format!("cannot write to database file at '{}' for restoring: {}",
                self.file_path, error.to_string()))
    }
}
