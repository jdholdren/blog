use crate::Result;
use crate::WithMessage;

use rusqlite::params;
use rusqlite::Connection;

pub struct Repo<'a> {
    pub conn: &'a Connection,
}

impl<'a> Repo<'a> {
    pub fn setup_tables(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE layouts (
            id TEXT PRIMARY KEY,
            html BLOB
        );",
            params![],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS blogposts (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            publish_date TEXT NOT NULL,
            html BLOB
        );",
            params![],
        )?;

        Ok(())
    }

    pub fn insert_blog(&self, blog: &Blog) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO blogposts (id, title, publish_date, html) VALUES (?1, ?2, ?3, ?4);",
                params![blog.id, blog.title, blog.publish_date, blog.html],
            )
            .with_context("issue inserting blog")?;

        Ok(())
    }

    pub fn insert_layout(&self, l: &Layout) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO layouts (id, html) VALUES (?1, ?2);",
                params![&l.id, &l.html],
            )
            .with_context("issue inserting layout")?;

        Ok(())
    }

    pub fn get_all_blogs(&self) -> Result<Vec<Blog>> {
        let mut stmt = self.conn.prepare("SELECT * FROM blogposts;")?;
        let iter = stmt.query_map([], Blog::from_row)?;

        let mut blogs = Vec::new();
        for blog in iter {
            blogs.push(blog?);
        }

        Ok(blogs)
    }

    pub fn get_layout(&self, name: &str) -> Result<Layout> {
        let mut stmt = self.conn.prepare("SELECT * FROM layouts WHERE id = ?;")?;
        let layout = stmt.query_row([name], Layout::from_row)?;

        Ok(layout)
    }
}

#[derive(Debug)]
pub struct Blog {
    pub id: String,
    pub title: String,
    pub publish_date: String,
    pub html: String,
}

impl Blog {
    fn from_row(row: &rusqlite::Row) -> std::result::Result<Blog, rusqlite::Error> {
        Ok(Blog {
            id: row.get(0)?,
            title: row.get(1)?,
            publish_date: row.get(2)?,
            html: row.get(3)?,
        })
    }
}

#[derive(Debug)]
pub struct Layout {
    pub id: String,
    pub html: String,
}

impl Layout {
    fn from_row(row: &rusqlite::Row) -> std::result::Result<Layout, rusqlite::Error> {
        Ok(Layout {
            id: row.get(0)?,
            html: row.get(1)?,
        })
    }
}
