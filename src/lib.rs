// DOCS

extern crate iron;

use iron::prelude::*;

use std::collections::HashMap;
use std::collections::hash_map;

enum Target {
    AfterMiddleware(Box<iron::middleware::AfterMiddleware>),
    Handler(Box<iron::Handler>),
}

impl iron::middleware::AfterMiddleware for Target {
    fn after(&self, req: &mut Request, res: Response) -> IronResult<Response> {
        match *self {
            Target::AfterMiddleware(ref x) => x.after(req, res),
            Target::Handler(ref x) => x.handle(req)
        }
    }

    fn catch(&self, req: &mut Request, err: IronError) -> IronResult<Response> {
        match *self {
            Target::AfterMiddleware(ref x) => x.catch(req, err),
            Target::Handler(ref x) => x.handle(req)
        }
    }
}

pub struct ErrorRouter {
    by_status: HashMap<iron::status::Status, Target>
}

impl Default for ErrorRouter {
    fn default() -> ErrorRouter { ErrorRouter { by_status: HashMap::new() } }
}

impl ErrorRouter {
    pub fn new() -> Self { ErrorRouter::default() }

    fn register(&mut self, status: iron::status::Status, target: Target) {
        match self.by_status.entry(status) {
            hash_map::Entry::Occupied(_) => panic!("Target for {:?} already registered.", status),
            hash_map::Entry::Vacant(x) => x.insert(target)
        };
    }

    pub fn handle_status<T: iron::Handler>(&mut self, status: iron::status::Status, handler: T) {
        self.register(status, Target::Handler(Box::new(handler)))
    }

    pub fn after_status<T: iron::middleware::AfterMiddleware>
        (&mut self, status: iron::status::Status, middleware: T) {
        self.register(status, Target::AfterMiddleware(Box::new(middleware)))
    }
}

impl iron::middleware::AfterMiddleware for ErrorRouter {
    fn after(&self, req: &mut Request, res: Response) -> IronResult<Response> {
        match (res.body.is_some(), res.status.and_then(|s| self.by_status.get(&s))) {
            (false, Some(x)) => x.after(req, res),
            _ => Ok(res)
        }
    }

    fn catch(&self, req: &mut Request, err: IronError) -> IronResult<Response> {
        match err.response.status.and_then(|s| self.by_status.get(&s)) {
            Some(x) => x.catch(req, err),
            None => Err(err)
        }
    }
}
