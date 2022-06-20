use std::fs;

use rusqlite::params;
use rusqlite::Connection;

// Defining a custom error message
#[derive(Debug)]
pub struct Error {
    context: String,
    inner_msg: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}: {}", self.context, self.inner_msg)
    }
}

impl std::error::Error for Error {}

impl From<rusqlite::Error> for Error {
    fn from(rusql_err: rusqlite::Error) -> Self {
        Error {
            context: String::new(),
            inner_msg: rusql_err.to_string(),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(io_err: std::io::Error) -> Self {
        Error {
            context: String::new(),
            inner_msg: io_err.to_string(),
        }
    }
}

pub trait WithMessage<T> {
    fn with_context(self, context: &str) -> Result<T, Error>;
}

impl<T, E: std::error::Error> WithMessage<T> for Result<T, E> {
    fn with_context(self, msg: &str) -> Result<T, Error> {
        match self {
            Ok(val) => Ok(val), // Just pass it through
            Err(err) => Err(Error {
                context: msg.to_string(),
                inner_msg: err.to_string(),
            }),
        }
    }
}

fn main() -> Result<(), Error> {
    // How I'm thinking about this going forward:
    //
    // I want to back this with a database for eventual search capability.
    // But largely this should all be statically generated; there's no need to
    // re-render the home page when nothing has changed.
    //
    // So first steps: parse files into a sqlite db, then start rendering pages
    // using queries from said database.

    // Build out the database
    let conn = Connection::open("./db.sqlite").with_context("issue opening db")?;

    // Crawl all static assets and insert them
    insert_static_entries(&conn)?;

    // Crawl all blogposts to insert into the db
    insert_blogs(&conn)?;

    Ok(())
}

// Inserts all static files into the db
fn insert_static_entries(conn: &Connection) -> Result<(), Error> {
    // Set up the table for reuseable assets
    conn.execute(
        "
        CREATE TABLE static_assets (
            id TEXT PRIMARY KEY,
            data BLOB
        );",
        params![],
    )?;

    let files = walk_directory("./static").with_context("error walking static dir")?;

    for file in files {
        // Read in the file and insert it into the db
        let contents =
            fs::read_to_string(&file).with_context(&format!("issue reading file: {}", file))?;

        conn.execute(
            "INSERT INTO static_assets (id, data) VALUES (?1, ?2)",
            params![&file, &contents],
        )?;
    }

    Ok(())
}

fn insert_blogs(conn: &Connection) -> Result<(), Error> {
    // Set up the table for blog posts
    conn.execute(
        "
        CREATE TABLE blogposts (
            id TEXT PRIMARY KEY,
            html BLOB
        );",
        params![],
    )?;

    for blog in walk_directory("./posts")? {
        let markdown =
            fs::read_to_string(&blog).with_context(&format!("error reading blog {}", blog))?;
        let parser = pulldown_cmark::Parser::new_ext(&markdown, pulldown_cmark::Options::all());

        // Insert it into the db
        let mut html = String::new();
        pulldown_cmark::html::push_html(&mut html, parser);

        conn.execute(
            "
             INSERT INTO blogposts (id, html) VALUES (?1, ?2);
         ",
            params![&blog, &html],
        )
        .with_context("issue inserting blog")?;
    }

    Ok(())
}

// Produces all files within a directory
fn walk_directory(dir: &str) -> Result<Vec<String>, Error> {
    let mut files: Vec<String> = vec![];

    for entry in fs::read_dir(dir).with_context("error reading dir")? {
        let entry = entry?;
        let path = entry.path().to_str().unwrap().to_string();

        // Skip directories
        if entry.metadata()?.is_dir() {
            // Delve into the directory
            files.append(&mut walk_directory(&path)?);
            continue;
        }

        // Skip .swp files
        if path.ends_with(".swp") {
            continue;
        }

        files.push(path);
    }

    Ok(files)
}
