use anyhow::{anyhow, Context, Result};

use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use maplit::hashmap;

#[derive(Debug)]
pub struct Templates(HashMap<String, String>);

impl Templates {
    pub fn new(m: HashMap<String, String>) -> Templates {
        Templates(m)
    }

    fn find(&self, id: &str) -> Result<&str> {
        match self.0.get(id) {
            Some(tpl) => Ok(tpl),
            None => Err(anyhow!("unable to find template named '{}'", id)),
        }
    }
}

#[derive(Debug)]
pub struct Blog {
    pub id: String,
    pub title: String,
    pub publish_date: String,
    pub excerpt: String,
    pub html: String,
    pub slug: String,
    pub external: Option<String>,
}

fn placeholder_func(f: &str, f_args: Vec<&str>, t_args: &HashMap<&str, String>) -> Result<String> {
    match f {
        "val" => {
            let arg_name = f_args[0];
            let value = match t_args.get(arg_name) {
                Some(v) => v,
                None => {
                    // This is a no-no: we are grabbing a value not in the template args
                    return Err(anyhow!("could not find argument: {}", arg_name));
                }
            };
            Ok(value.to_owned())
        }
        "opt" => {
            let value = if t_args.get(f_args[0]).unwrap_or(&String::new()) == "true" {
                f_args[1]
            } else {
                f_args[2]
            };

            // Need to parse the optional arguments
            Ok(value.to_owned())
        }
        s => Err(anyhow!("unrecognized function: {}", s)),
    }
}

// The struct that can render pages given templates and blogs
pub struct Pages<'a> {
    templates: Templates,
    repleaceables: Templates,
    blogs: Vec<Blog>,
    page_list: Vec<&'a str>,
}

impl<'a> Pages<'a> {
    pub fn new(mut blogs: Vec<Blog>, templates: Templates, repleaceables: Templates) -> Pages<'a> {
        blogs.sort_by(|a, b| b.publish_date.partial_cmp(&a.publish_date).unwrap());

        Pages {
            page_list: vec![],
            blogs,
            templates,
            repleaceables,
        }
    }

    // Adds the reusable templates to the argments going to any layout
    fn add_replacebales(
        &'a self,
        mut args: HashMap<&'a str, String>,
    ) -> Result<HashMap<&str, String>> {
        for (id, template) in &self.repleaceables.0 {
            let replaced = self
                .replace_placeholders(template, &args)
                .with_context(|| format!("replacing for template: {}", id))?;
            args.insert(id, replaced);
        }

        Ok(args)
    }

    fn replace_placeholders<'b>(
        &'a self,
        layout: &str,
        args: &HashMap<&'b str, String>,
    ) -> Result<String> {
        let mut result = layout.to_owned();

        // Cause we need to do this in reverse order...
        let mut v = Vec::new();

        // Can use a lazy macro to make this static
        let re = Regex::new(r"\{\{([a-z_]+)\(([a-z\\,_]+)\)\}\}").unwrap();
        let caps = re.captures_iter(&result);
        for cap in caps {
            let outer_group = cap.get(0).unwrap();
            let (start, end) = (outer_group.start(), outer_group.end());

            let fn_name = &cap[1];
            let fn_args: Vec<&str> = cap[2].split(',').collect();

            v.push((placeholder_func(fn_name, fn_args, args)?, start, end));
        }
        v.reverse();

        for (value, start, end) in v {
            result.replace_range(start..end, &value);
        }

        Ok(result)
    }

    // Template generators should have to call this, not the other two functions
    fn render<'b>(&'a self, layout: &str, args: HashMap<&'b str, String>) -> Result<String> {
        let layout_args = self.add_replacebales(args)?;
        self.replace_placeholders(layout, &layout_args)
    }

    pub fn generate_index(&mut self) -> Result<()> {
        let blogs = &self.blogs[0..3];

        let mut blogs_arg = String::new();
        for blog in blogs {
            let blurb = self.blog_to_blurb(blog)?;
            blogs_arg.push_str(&blurb);
        }

        // Need the template for the page
        let layout = self.templates.find("index")?;

        let mut args: HashMap<&str, String> = HashMap::new();
        args.insert("latest_posts", blogs_arg);

        let contents = self.render(layout, args)?;
        let mut f = File::create("./generated/index.html")?;
        f.write_all(contents.as_bytes())?;

        // Add the page for the sitelist
        // TODO: Don't hardcode this? eh idk
        self.page_list.push("https://jamesholdren.com");

        Ok(())
    }

    pub fn generate_all_posts(&mut self) -> Result<()> {
        // Need all the blogs to render to a blurb list
        let blogs = &self.blogs;

        let mut blogs_arg = String::new();
        for blog in blogs {
            let blurb = self.blog_to_blurb(blog)?;
            blogs_arg.push_str(&blurb);
        }

        // Need the template for the page
        let layout = self.templates.find("all_posts")?;

        let mut args: HashMap<&str, String> = HashMap::new();
        args.insert("posts", blogs_arg);
        args.insert("posts_selected", String::from("true"));

        let contents = self
            .render(layout, args)
            .context("issue replacing placeholders on all blogs page")?;
        std::fs::create_dir_all("./generated/posts")?;
        let mut f = File::create("./generated/posts/index.html")?;
        f.write_all(contents.as_bytes())?;

        // Add page to sitemap
        self.page_list.push("https://jamesholdren.com/posts");

        // For each blog post, generate its page
        for blog in blogs.iter().filter(|blog| blog.external.is_none()) {
            let layout = self.templates.find("post")?;
            let contents = self.render(
                layout,
                hashmap! {
                    "title" => blog.title.to_owned(),
                    "description" => blog.excerpt.to_owned(),
                    "publish_date" => blog.publish_date.to_owned(),
                    "contents" => blog.html.to_owned(),
                },
            )?;
            std::fs::create_dir_all(format!("./generated/posts/{}", blog.slug))?;
            let mut f = File::create(format!("./generated/posts/{}/index.html", blog.slug))?;
            f.write_all(contents.as_bytes())?;
        }

        Ok(())
    }

    // Converts a blog post to a short excerpt string
    fn blog_to_blurb(&self, b: &Blog) -> Result<String> {
        // Get the layout for the blurb
        let layout = self
            .templates
            .find("blurb")
            .context("finding blurb from repleaceables")?;

        // Use the external link or slug it out
        let link = b
            .external
            .to_owned()
            .unwrap_or(format!("/posts/{}/", b.slug));

        self.render(
            layout,
            hashmap! {
                "title" => b.title.to_owned(),
                "excerpt" => b.excerpt.to_owned(),
                "publish_date" => b.publish_date.to_owned(),
                "link" => link,
            },
        )
    }

    // Takes the page_list and turns it into a text sitemap
    pub fn generate_sitemap(&self) -> Result<()> {
        let mut f = File::create("./generated/sitemap.txt")?;
        for page in &self.page_list {
            write!(f, "{}", page)?;
        }

        Ok(())
    }
}
