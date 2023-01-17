pub struct Report {
    headers: Vec<&'static str>,
    headers_max_len: Vec<usize>,
    items: Vec<Vec<String>>
}

impl Report {
    pub fn new(headers: Vec<&'static str>) -> Report {
        let headers_len = headers.len();
        Report { headers, headers_max_len: vec![0; headers_len], items: Vec::new() }
    }

    pub fn add_item(&mut self, item: Vec<String>) {
        assert_eq!(item.len(), self.headers.len());

        for (index, headers_max_len) in self.headers_max_len.iter_mut().enumerate() {
            let item_len = item.get(index).unwrap().len();
            if *headers_max_len < item_len {
                *headers_max_len = item_len;
            }
        }

        self.items.push(item);
    }

    pub fn show(self, trim_whitespace: bool) {
        for item in self.items {
            let mut buffer: Vec<String> = Vec::new();
            for (index, column) in item.iter().enumerate() {
                let column_text = if trim_whitespace {
                    format!("{column}")
                } else {
                    format!("{:1$}", column, self.headers_max_len[index])
                };
                buffer.push(column_text);
            }

            println!("{}", buffer.join(" "));
        }
    }
}

pub mod client_report;
