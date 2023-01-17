use std::{collections::HashMap, path::PathBuf};
use chrono::{Utc, Duration, DateTime};
use crate::db::{
    Database, Client, Payment, Target,
    datetime_serializer::{datetime_to_str, datetime_from_str}
};
use rusqlite::{Connection, Transaction};

macro_rules! try_sql {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(error) => {
                return Err(format!("sql error: {}", error.to_string()));
            }
        } 
    };
}

pub struct SqliteDb<'a> {
    trans: Transaction<'a>
}

impl<'a> SqliteDb<'a> {
    pub fn create_connection(db_path: PathBuf) -> Result<Connection, String> {
        Connection::open(db_path).map_err(|e| e.to_string())
    }

    pub fn new(conn: &'a mut Connection) -> Result<Self, String> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS clients (
                name TEXT PRIMARY KEY,
                expire_date TEXT NOT NULL,
                info TEXT
            )", ()
        ).map_err(|e| format!("cannot create clients table: {}", e.to_string()))?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS payments (
                client_name TEXT NOT NULL,
                seller TEXT NOT NULL,
                date TEXT NOT NULL,
                money UNSIGNED INTEGER NOT NULL
            )", ()
        ).map_err(|e| format!("cannot create payments table: {}", e.to_string()))?;
        Ok(SqliteDb { trans: conn.transaction().map_err(|e| e.to_string())? })
    }

    fn get_payments(&self) -> Result<HashMap<String, Vec<Payment>>, String> {
        let mut stmt = try_sql!(self.trans.prepare(
            "SELECT client_name, seller, date, money FROM payments",
        ));
        let mut rows = try_sql!(stmt.query([]));

        let mut payments: HashMap<String, Vec<Payment>> = HashMap::new();
        while let Some(row) = try_sql!(rows.next()) {
            let client_name: String = try_sql!(row.get(0));
            let date: String = try_sql!(row.get(2));

            let payment = Payment {
                seller: try_sql!(row.get(1)),
                date: datetime_from_str(&date),
                money: try_sql!(row.get(3))
            };

            payments.entry(client_name).and_modify(|v| v.push(payment.clone()))
                .or_insert(vec![payment]);
        }
        
        Ok(payments)
    }

    fn add_payment(&mut self, client_name: &str, seller: &str, date: &str, money: u32) -> Result<(), String> {
        try_sql!(self.trans.execute(
            "INSERT INTO payments (client_name, seller, date, money) VALUES (?, ?, ?, ?)",
            (client_name, seller, date, money)
        ));
        Ok(())
    }

    fn get_client_expire_date(&self, client_name: &str) -> Result<DateTime<Utc>, String> {
        let mut stmt = try_sql!(self.trans.prepare("SELECT expire_date FROM clients WHERE name=? LIMIT 1"));
        let mut rows = try_sql!(stmt.query([client_name]));
        let expire_date = match try_sql!(rows.next()) {
            Some(row) => {
                let expire_date: String = try_sql!(row.get(0));
                Ok(expire_date)
            },
            None => Err(format!("client with name '{}' doesn't exists!", client_name))
        }?;

        Ok(datetime_from_str(&expire_date))
    }
}

impl Database for SqliteDb<'_> {
    fn add_client(&mut self, name: &str, days: u32, seller: &str, money: u32, info: &str) -> Result<(), String> {
        let new_client = Client::new(name, days, &seller, money, info);
        let expire_date = datetime_to_str(&new_client.expire_time);
        let payment_date = datetime_to_str(&new_client.payments.get(0).unwrap().date);

        let rows_affected = try_sql!(self.trans.execute(
            "INSERT OR IGNORE INTO clients (name, expire_date, info) VALUES (?, ?, ?)",
            (name, expire_date.as_str(), info)
        ));

        if rows_affected == 0 {
            return Err(format!("client '{}' already exists!", name))
        }

        self.add_payment(name, seller, &payment_date, money)?;
        Ok(())
    }

    fn renew_client(&mut self, name: &str, days: u32, seller: &str, money: u32) -> Result<(), String> {
        let mut expire_date = self.get_client_expire_date(name)?;
        let now_date = Utc::now();
        if now_date > expire_date {
            expire_date = now_date;
        }
        expire_date += Duration::days(days.into());

        let rows_affected = try_sql!(
            self.trans.execute(
                "UPDATE clients SET expire_date=? WHERE name=?",
                (datetime_to_str(&expire_date), name)
            )
        );
        assert!(rows_affected > 0);
        self.add_payment(name, seller, &datetime_to_str(&now_date), money)?;
        Ok(())
    }

    fn renew_all_clients(&mut self, days: u32) -> Result<(), String> {
        let mut stmt = try_sql!(self.trans.prepare("SELECT name, expire_date FROM clients"));
        let rows = try_sql!(stmt.query_map([], |row| {
            let name: String = row.get(0)?;
            let expire_date: String = row.get(1)?;
            Ok((name, datetime_from_str(&expire_date)))
        }));

        let now_date = Utc::now();
        for row in rows {
            let (name, mut expire_date) = try_sql!(row);
            if expire_date < now_date {
                continue;
            }
            expire_date += Duration::days(days.into());

            let rows_affected = try_sql!(
                self.trans.execute(
                    "UPDATE clients SET expire_date=? WHERE name=?",
                    (datetime_to_str(&expire_date), name)
                )
            );
            assert!(rows_affected > 0);
        }
        Ok(())
    }

    fn remove_client(&mut self, name: &str) -> Result<(), String> {
        let rows_affected = try_sql!(self.trans.execute("DELETE FROM clients WHERE name=?", (name, )));
        if rows_affected == 0 {
            return Err(format!("client with name '{}' doesn't exists!", name))
        }

        assert!(try_sql!(self.trans.execute("DELETE FROM payments WHERE client_name=?", (name, ))) > 0);
        Ok(())
    }

    fn list_clients(&self) -> Result<Vec<Client>, String> {
        let mut payments = self.get_payments()?;

        let mut stmt = try_sql!(self.trans.prepare("SELECT name, expire_date, info FROM clients"));
        let mut rows = try_sql!(stmt.query([]));

        let mut clients: Vec<Client> = Vec::new();
        while let Some(row) = try_sql!(rows.next()) {
            let client_name: String = try_sql!(row.get(0));
            let expire_date: String = try_sql!(row.get(1));

            clients.push(Client {
                payments: payments.remove(&client_name).unwrap(),
                name: client_name,
                expire_time: datetime_from_str(&expire_date),
                info: try_sql!(row.get(2))
            });
        }

        Ok(clients)
    }

    fn rename_client(&mut self, old_name: &str, new_name: &str) -> Result<(), String> {
        let rows_affected = try_sql!(self.trans.execute("UPDATE clients SET name=? WHERE name=?", (new_name, old_name)));
        if rows_affected == 0 {
            return Err(format!("client with name '{}' doesn't exists!", old_name))
        }

        let rows_affected = try_sql!(
            self.trans.execute(
                "UPDATE payments SET client_name=? WHERE client_name=?",
                (new_name, old_name)
            )
        );
        assert!(rows_affected > 0);
        Ok(())
    }

    fn set_client_info(&mut self, target: Target, info: &str) -> Result<(), String> {
        let stmt = match target {
            Target::All => self.trans.execute("UPDATE clients SET info=?", (info, )),
            Target::MatchInfo(ref old_info) => self.trans.execute("UPDATE clients SET info=? WHERE info=?", (info, old_info)),
            Target::OnePerson(ref name) => self.trans.execute("UPDATE clients SET info=? WHERE name=?", (info, name)),
        };

        let rows_affected = try_sql!(stmt);
        if let Target::OnePerson(name) = target {
            if rows_affected == 0 {
                return Err(format!("client with name '{}' doesn't exists!", name));
            } 
        }

        Ok(())
    }

    fn get_client_info(&self, name: &str) -> Result<String, String> {
        let mut stmt = try_sql!(self.trans.prepare("SELECT info FROM clients WHERE name=? LIMIT 1"));
        let mut rows = try_sql!(stmt.query([name]));
        
        let maybe_row = try_sql!(rows.next());
        match maybe_row {
            Some(row) => Ok(try_sql!(row.get(0))),
            None => Err(format!("client with name '{}' doesn't exists!", name))
        }
    }

    fn commit(self) -> Result<(), String> {
        try_sql!(self.trans.commit());
        Ok(())
    }
}
