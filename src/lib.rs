// DOCS

extern crate iron;

use iron::prelude::*;
use iron::modifier::Modifier;

use std::collections::HashMap;
use std::collections::hash_map;

trait ModifierInner: Send + Sync {
    fn clone_box(&self) -> Box<ModifierInner>;
    fn apply_modify(self: Box<Self>, res: &mut Response);
}

impl<M: Modifier<Response> + Send + Sync + Clone + 'static> ModifierInner for M {
    fn clone_box(&self) -> Box<ModifierInner> {
        Box::new(self.clone())
    }

    fn apply_modify(self: Box<Self>, res: &mut Response) { self.modify(res) }
}

impl Modifier<Response> for Box<ModifierInner> {
    fn modify(self, res: &mut Response) { self.apply_modify(res) }
}

enum Target {
    AfterMiddleware(Box<iron::middleware::AfterMiddleware>),
    Modifier(Box<ModifierInner>),
    Handler(Box<iron::Handler>),
}

impl iron::middleware::AfterMiddleware for Target {
    fn after(&self, req: &mut Request, res: Response) -> IronResult<Response> {
        match *self {
            Target::AfterMiddleware(ref x) => x.after(req, res),
            Target::Modifier(ref x) => Ok(res.set(x.clone_box())),
            Target::Handler(ref x) => x.handle(req)
        }
    }

    fn catch(&self, req: &mut Request, mut err: IronError) -> IronResult<Response> {
        match *self {
            Target::AfterMiddleware(ref x) => x.catch(req, err),
            Target::Modifier(ref x) => { err.response.set_mut(x.clone_box()); Err(err) },
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

    pub fn modifier_for_status<T: Modifier<Response> + Send + Sync + Clone + 'static>
        (&mut self, status: iron::status::Status, modifier: T) {
        self.register(status, Target::Modifier(Box::new(modifier)))
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
