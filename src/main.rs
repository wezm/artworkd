extern crate iron;
extern crate logger;
#[macro_use]
extern crate router;
extern crate image;

use iron::{Iron, Request, Response, IronResult, IronError, Chain};
use iron::mime::Mime;
use iron::status;
use router::{Router};

use logger::Logger;
use std::path::{PathBuf};
use std::fs;
use std::io;

fn hello_world(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((iron::status::Ok, "Hi!")))
}

fn artwork(req: &mut Request) -> IronResult<Response> {
    let mut filepath = PathBuf::from("/Users/wmoore/Projects/Musicast/web/artwork");

    let ref variant = req.extensions.get::<Router>().unwrap().find("variant").unwrap();
    let ref subdir = req.extensions.get::<Router>().unwrap().find("subdir").unwrap();
    let ref file = req.extensions.get::<Router>().unwrap().find("file").unwrap();

    // TODO: Sanitize components before constructing a path from them

    if *variant != "thumb" {
        return Ok(Response::with((iron::status::NotFound, "Not Found: Unknown variant")));
    }

    filepath.push(subdir);
    filepath.push(file);

    let metadata = match fs::metadata(&filepath) {
        Ok(meta) => meta,
        Err(e) => {
            let status = match e.kind() {
                io::ErrorKind::NotFound => status::NotFound,
                io::ErrorKind::PermissionDenied => status::Forbidden,
                _ => status::InternalServerError,
            };

            return Err(IronError::new(e, status))
        },
    };

    let img = match image::open(&filepath) {
        Ok(img) => img,
        Err(e) => return Err(IronError::new(e, status::InternalServerError))
    };

    let thumb = img.resize(128, 128, image::FilterType::Triangle);
    let mut buffer = vec![];

    match thumb.save(&mut buffer, image::JPEG) {
        Ok(_) => {
            let content_type = "image/jpeg".parse::<Mime>().unwrap();
            Ok(Response::with((content_type, iron::status::Ok, buffer)))
        },
        Err(e) => Err(IronError::new(e, status::InternalServerError))
    }
}

fn main() {
    let router = router!(get "/" => hello_world,
                         get "/:variant/:subdir/:file" => artwork);

    let (logger_before, logger_after) = Logger::new(None);
    let mut chain = Chain::new(router);
    chain.link_before(logger_before);

    // Add other middleware here

    chain.link_after(logger_after);
    Iron::new(chain).http("localhost:3004").unwrap();
}

