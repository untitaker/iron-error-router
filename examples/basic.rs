extern crate iron;
extern crate iron_error_router;

use iron::prelude::*;
use iron::status;

fn main() {
    let handler = |_: &mut Request| {
        Ok(Response::with((status::NotFound)))
    };

    let mut chain = Chain::new(handler);
    chain.link_after({
        let mut error_router = iron_error_router::ErrorRouter::new();
        error_router.handle_status(status::NotFound, |_: &mut Request| {
            // If you need to use the original response, you can also register an AfterMiddleware
            // using `error_router.after_status`.
            Ok(Response::with((
                status::NotFound,
                "Content not found. Check if you have an internet connection."  // Totally useful error page that is not irritating anybody.
            )))
        });
        error_router
    });

    Iron::new(chain).http("localhost:3000").unwrap();
}
