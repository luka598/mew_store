use std::collections::HashMap;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case", tag = "op")]
pub enum Operation {
    Get {
        key: String,
    },
    Set {
        key: String,
        value: String,
    },
    Del {
        key: String,
    },
    List {
        pattern: String,
        cursor: u64,
        limit: u64,
    },
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Request {
    None,
    Exec {
        op: Operation,
        extra: HashMap<String, String>,
    },
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Response {
    None,
    Error(String),
    Values(Vec<String>),
}

pub struct Store {
    middleware: Vec<Box<dyn Middleware>>,
}

impl Store {
    pub fn new() -> Self {
        Self { middleware: Vec::new() }
    }

    pub fn add_middleware(mut self, mw: impl Middleware + 'static) -> Self {
        self.middleware.push(Box::new(mw));
        self
    }

    pub fn run(&self, mut req: Request) -> Response {
        let mut depth: usize = 0;
        let mut response = None;

        while depth < self.middleware.len() {
            match self.middleware[depth].before(req) {
                Flow::Next(new_req) => {
                    req = new_req;
                    depth += 1;
                }
                Flow::Return(res) => {
                    response = Some(res);
                    break;
                }
            }
        }

        let mut response = response.unwrap_or_else(|| Response::Error("no_return".to_string()));

        while depth > 0 {
            depth -= 1;
            response = self.middleware[depth].after(response);
        }

        response
    }

}

pub enum Flow {
    Next(Request),
    Return(Response),
}

pub trait Middleware: Send + Sync {
    fn before(&self, req: Request) -> Flow {
        Flow::Next(req)
    }

    fn after(&self, res: Response) -> Response {
        res
    }
}