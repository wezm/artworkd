extern crate iron;
extern crate logger;
#[macro_use]
extern crate router;

extern crate image_middleware;

use iron::{Iron, Request, Response, IronResult, Chain};

use logger::Logger;

// This might be useful for showing stats, config, etc. Kinda like phpinfo but
// prettier and with security considerations
fn info(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((iron::status::Ok, "artworkd v0.1.0")))
}

fn main() {
    let router = router!(get "/" => info,
                         get "/:variant/:subdir/:file" => image_middleware::artwork);

    let (logger_before, logger_after) = Logger::new(None);
    let mut chain = Chain::new(router);
    chain.link_before(logger_before);

    // Add other middleware here

    chain.link_after(logger_after);
    Iron::new(chain).http("localhost:3004").unwrap();
}

