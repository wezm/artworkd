extern crate iron;
extern crate logger;
#[macro_use]
extern crate router;
extern crate image;

use iron::{Iron, Request, Response, IronResult, IronError, Chain};
use iron::status;
use router::{Router};

use logger::Logger;
use std::path::{PathBuf};
use std::fs;
use std::io;
use std::io::Cursor;
use std::fs::File;

use image::GenericImage;

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
    // let jpg_thumb: Vec<u8> = Vec::new();
    // let cursor = Cursor::new(&mut jpg_thumb);

    let mut thumbpath = filepath.clone();
    let new_file_name = format!("{}.{}", thumbpath.file_name().unwrap().to_string_lossy(), variant);
    thumbpath.set_file_name(new_file_name);
    let (width, height) = thumb.dimensions();
    println!("File ({}): {}, {}x{} -> {}", variant, filepath.to_string_lossy(), width, height, thumbpath.to_string_lossy());

    let ref mut thumb_file = match File::create(&thumbpath) {
        Ok(f) => f,
        Err(e) => return Err(IronError::new(e, status::InternalServerError))
    };

    match thumb.save(thumb_file, image::JPEG) {
        Ok(_) => Ok(Response::with((iron::status::Ok, thumbpath))),
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

