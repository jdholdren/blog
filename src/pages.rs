use anyhow::{Context, Result};
use chrono::NaiveDate;
use maplit::hashmap;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

mod layout;

pub use layout::Renderer;

#[derive(Debug)]
pub struct Blog {
    pub id: String,
    pub title: String,
    pub publish_date: NaiveDate,
    pub display_date: String,
    pub excerpt: String,
    pub html: String,
    pub slug: String,
    pub external: Option<String>,
}

// The struct that can render pages given templates and blogs
pub struct Pages {
    renderer: layout::Renderer,
    blogs: Vec<Blog>,
    page_list: Vec<String>,
}

impl Pages {
    pub fn new(mut blogs: Vec<Blog>, renderer: layout::Renderer) -> Self {
        blogs.sort_by(|a, b| b.publish_date.partial_cmp(&a.publish_date).unwrap());

        Pages {
            page_list: vec![],
            blogs,
            renderer,
        }
    }

    pub fn generate_index(&mut self) -> Result<()> {
        let blogs = &self.blogs[0..3];

        let mut blogs_arg = String::new();
        for blog in blogs {
            let blurb = self.blog_to_blurb(blog)?;
            blogs_arg.push_str(&blurb);
        }

        let mut args: HashMap<&str, String> = HashMap::new();
        args.insert("latest_posts", blogs_arg);

        let contents = self.renderer.render_layout("index", &args)?;
        let mut f = File::create("./generated/index.html")?;
        f.write_all(contents.as_bytes())?;

        // Add the page for the sitelist
        // TODO: Don't hardcode this? eh idk
        self.page_list.push("https://jamesholdren.com".to_string());

        Ok(())
    }

    pub fn generate_all_posts(&mut self) -> Result<()> {
        let mut blogs_arg = String::new();
        for blog in &self.blogs {
            let blurb = self.blog_to_blurb(blog)?;
            blogs_arg.push_str(&blurb);
        }

        let contents = self
            .renderer
            .render_layout(
                "all_posts",
                &hashmap! {
                    "posts" => blogs_arg,
                    "posts_selected" => String::from("true"),
                },
            )
            .context("issue replacing placeholders on all blogs page")?;
        std::fs::create_dir_all("./generated/posts")?;
        let mut f = File::create("./generated/posts/index.html")?;
        f.write_all(contents.as_bytes())?;

        // Add page to sitemap
        self.page_list
            .push("https://jamesholdren.com/posts/".to_string());

        // For each blog post, generate its page
        for blog in self.blogs.iter().filter(|blog| blog.external.is_none()) {
            let contents = self.renderer.render_layout(
                "post",
                &hashmap! {
                    "title" => blog.title.to_owned(),
                    "description" => blog.excerpt.to_owned(),
                    "publish_date" => blog.display_date.to_owned(),
                    "contents" => blog.html.to_owned(),
                },
            )?;
            std::fs::create_dir_all(format!("./generated/posts/{}/", blog.slug))?;
            let mut f = File::create(format!("./generated/posts/{}/index.html", blog.slug))?;
            f.write_all(contents.as_bytes())?;

            // Add the post to the sitemap
            self.page_list
                .push(format!("https://jamesholdren.com/posts/{}/", blog.slug));
        }

        Ok(())
    }

    // Converts a blog post to a short excerpt string
    fn blog_to_blurb(&self, b: &Blog) -> Result<String> {
        // Use the external link or slug it out
        let link = b
            .external
            .to_owned()
            .unwrap_or(format!("/posts/{}/", b.slug));

        self.renderer.render_layout(
            "blurb",
            &hashmap! {
                "title" => b.title.to_owned(),
                "excerpt" => b.excerpt.to_owned(),
                "publish_date" => b.display_date.to_owned(),
                "link" => link,
            },
        )
    }

    // Takes the page_list and turns it into a text sitemap
    pub fn generate_sitemap(&self) -> Result<()> {
        let mut f = File::create("./generated/sitemap.txt")?;
        for page in &self.page_list {
            writeln!(f, "{page}")?;
        }

        Ok(())
    }
}
