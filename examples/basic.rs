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

        // In case the response is a 404, replace the response with your own.

        error_router.handle_status(status::NotFound, |_: &mut Request| {
            Ok(Response::with((
                status::NotFound,
                "Content not found."
            )))
        });

        // Instead of writing a handler, you can just use a modifier:

        error_router.modifier_for_status(
            status::NotFound,
            (status::NotFound, "Content not found.")
        );

        // Or an AfterMiddleware:

        error_router.after_status(status::NotFound, |_: &mut Request, _: Response| {
            Ok(Response::with((status::NotFound, "Content not found.")))
        });

        error_router
    });

    Iron::new(chain).http("localhost:3000").unwrap();
}
