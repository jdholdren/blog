use std::collections::HashMap;
use std::fs;
use std::io::Cursor;

use pulldown_cmark::Parser;
use rusqlite::params;
use rusqlite::Connection;

// Defining a custom error message
#[derive(Debug)]
pub struct Error {
    context: String,
    inner_msg: String,
}

impl Error {
    fn new(msg: &str) -> Error {
        Error {
            context: String::new(),
            inner_msg: msg.to_string(),
        }
    }
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

    // Crawl all layouts and put them in the DB
    insert_layouts(&conn)?;

    // Crawl all static assets and insert them
    // insert_static_entries(&conn)?;

    // Crawl all blogposts to insert into the db
    insert_blogs(&conn)?;

    Ok(())
}

const LAYOUT_DIR: &str = "./layouts/";

fn insert_layouts(conn: &Connection) -> Result<(), Error> {
    conn.execute(
        "
        CREATE TABLE layouts (
            id TEXT PRIMARY KEY,
            html BLOB
        );",
        params![],
    )?;

    for mut layout in walk_directory(LAYOUT_DIR)? {
        let contents = fs::read_to_string(&layout)?;

        // Trim off the folder name to get the layout id
        layout.replace_range(0..LAYOUT_DIR.len(), "");

        // Insert it into the db
        conn.execute(
            "INSERT INTO layouts (id, html) VALUES (?1, ?2);",
            params![&layout, &contents],
        )
        .with_context("issue inserting layout")?;
    }

    Ok(())
}

fn insert_blogs(conn: &Connection) -> Result<(), Error> {
    // Set up the table for blog posts
    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS blogposts (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            publish_date TEXT NOT NULL,
            html BLOB
        );",
        params![],
    )?;

    for blog in walk_directory("./posts")? {
        let contents = fs::read_to_string(&blog)?;
        let mut parser = pulldown_cmark::Parser::new_ext(&contents, pulldown_cmark::Options::all());

        // Meta information about the blog
        let front_matter = parse_frontmatter(&mut parser)
            .with_context(&format!("could not parse frontmatter for {}", blog))?;
        let metadata = frontmatter_to_meta(&front_matter);

        let mut bytes = Vec::new();
        pulldown_cmark::html::write_html(Cursor::new(&mut bytes), parser)?;
        let html = &String::from_utf8_lossy(&bytes)[..];

        // Insert it into the db
        conn.execute(
            "INSERT INTO blogposts (id, title, publish_date, html) VALUES (?1, ?2, ?3, ?4);",
            params![&blog, metadata.title, metadata.publish_date, &html],
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

type Frontmatter = HashMap<String, String>;

// Parses a md file to get the front matter and the offset of the
// real content
fn parse_frontmatter(parser: &mut Parser) -> Result<Frontmatter, Error> {
    let mut reading_frontmatter = false;
    let mut fm = Frontmatter::new();

    'events: for event in parser {
        if !reading_frontmatter {
            match event {
                pulldown_cmark::Event::Rule => {
                    reading_frontmatter = true; // start parsing
                    continue;
                }
                pulldown_cmark::Event::SoftBreak => {}
                _ => {
                    return Err(Error::new("frontmatter not found"));
                }
            }
        }

        let data = match event {
            pulldown_cmark::Event::Text(tag) => tag.to_string(), // Info
            pulldown_cmark::Event::Rule => break 'events,        // End of the frontmatter
            pulldown_cmark::Event::SoftBreak => continue,        // Ignored
            pulldown_cmark::Event::HardBreak => continue,        // Ignored
            pulldown_cmark::Event::Start(_) => continue,         // Ignored
            pulldown_cmark::Event::End(_) => continue,           // Ignored
            _ => {
                // Unhandle-able
                return Err(Error::new(&format!("bad frontmatter, found: {:?}", event)));
            }
        };

        // The first delimeter is a ':', then everything after is the value
        let mut split = data.split(":").collect::<Vec<&str>>();
        let key = split[0].to_string();
        split.remove(0);
        let value = split.join("").trim().to_string();

        // New we're good to insert into our fm
        fm.insert(key, value);
    }

    Ok(fm)
}

struct Meta {
    title: String,
    publish_date: String,
}

fn frontmatter_to_meta(fm: &Frontmatter) -> Meta {
    Meta {
        title: fm.get("title").unwrap_or(&"".to_string()).to_string(),
        publish_date: fm.get("publishDate").unwrap_or(&"".to_string()).to_string(),
    }
}
