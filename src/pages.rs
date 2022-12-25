use crate::Error;
use crate::Result;

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
            Some(tpl) => Ok(&tpl),
            None => Err(Error::new(&format!(
                "unable to find template named '{}'",
                id
            ))),
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
}

pub struct Pages<'a> {
    templates: Templates,
    blogs: Vec<Blog>,
    page_list: Vec<&'a str>,
}

impl<'a> Pages<'a> {
    pub fn new(mut blogs: Vec<Blog>, templates: Templates) -> Pages<'a> {
        blogs.sort_by(|a, b| a.publish_date.partial_cmp(&b.publish_date).unwrap());

        Pages {
            page_list: vec![],
            blogs,
            templates,
        }
    }

    // Adds the reusable templates to the argments going to any layout
    fn add_tpls(&'a self, mut args: HashMap<&'a str, &'a str>) -> HashMap<&str, &str> {
        for (id, template) in &self.templates.0 {
            args.insert(&id, &template);
        }

        args
    }

    pub fn generate_index(&mut self) -> Result<()> {
        let blogs = &self.blogs[0..3];

        let mut blogs_arg = String::new();
        for blog in blogs {
            let blurb = self.blog_to_blurb(&blog)?;
            blogs_arg.push_str(&blurb);
        }

        // Need the template for the page
        let layout = self.templates.find("index")?;

        let mut args: HashMap<&str, &str> = HashMap::new();
        args.insert("latest_posts", &blogs_arg);

        let contents = replace_placeholders(&layout, self.add_tpls(args))?;
        let mut f = File::create("./generated/index.html")?;
        f.write_all(contents.as_bytes())?;

        // Add the page for the sitelist
        // TODO: Don't hardcode this? eh idk
        self.page_list.push("https://jamesholdren.com");

        Ok(())
    }

    pub fn generate_all_posts(&mut self) -> Result<()> {
        // Need all the blogs to render to a list
        let blogs = &self.blogs;

        let mut blogs_arg = String::new();
        for blog in blogs {
            let blurb = self.blog_to_blurb(&blog)?;
            blogs_arg.push_str(&blurb);
        }

        // Need the template for the page
        let layout = self.templates.find("all_posts")?;

        let mut args: HashMap<&str, &str> = HashMap::new();
        args.insert("posts", &blogs_arg);

        let contents = replace_placeholders(&layout, self.add_tpls(args))?;
        std::fs::create_dir_all("./generated/posts")?;
        let mut f = File::create("./generated/posts/index.html")?;
        f.write_all(contents.as_bytes())?;

        // Add page to sitemap
        self.page_list.push("https://jamesholdren.com/posts");

        Ok(())
    }

    // Converts a blog post to a short excerpt string
    fn blog_to_blurb(&self, b: &Blog) -> Result<String> {
        // Get the layout for the blurb
        let layout = self.templates.find("blurb")?;
        replace_placeholders(
            &layout,
            hashmap! {
                "title" => b.title.as_str(),
                "excerpt" => b.excerpt.as_str(),
                "publish_date" => b.publish_date.as_str(),
                "link" => b.slug.as_str(),
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

fn replace_placeholders(layout: &str, args: HashMap<&str, &str>) -> Result<String> {
    let mut result = layout.to_owned();

    // Cause we need to do this in reverse order...
    let mut v = Vec::new();

    // Can use a lazy macro to make this static
    let re = Regex::new(r"\{\{([a-z._]+)\}\}").unwrap();
    let caps = re.captures_iter(&result);
    for cap in caps {
        let outer_group = cap.get(0).unwrap();
        let (start, end) = (outer_group.start(), outer_group.end());

        let inner_group = cap
            .get(1)
            .map_or(String::from(""), |f| f.as_str().to_owned());

        v.push((inner_group, start, end));
    }
    v.reverse();

    for (name, start, end) in v {
        // First see if we have that value in `args`
        let value = match args.get(&*name) {
            Some(val) => val,
            None => return Err(Error::new(&format!("could not find argument: {}", name))),
        };

        result.replace_range(start..end, value);
    }

    Ok(result)
}
