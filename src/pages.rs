use crate::repo;
use crate::Error;
use crate::Result;
use repo::{Blog, Repo};

use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use maplit::hashmap;

pub struct Pages<'a> {
    pub repo: &'a Repo<'a>,
}

impl<'a> Pages<'a> {
    pub fn generate_index(&self) -> Result<()> {
        // Need all the blogs to render to a list
        let blogs = self.repo.get_all_blogs()?;

        let mut blogs_arg = String::new();
        for blog in blogs {
            let blurb = self.blog_to_blurb(&blog)?;
            blogs_arg.push_str(&blurb);
        }

        // Need the template for the page
        let layout = self.repo.get_layout("index.layout.html")?;

        let mut args: HashMap<&str, &str> = HashMap::new();
        args.insert("blurbs", &blogs_arg);

        let contents = replace_placeholders(&layout.html, args)?;
        let mut f = File::create("./generated/index.html")?;
        f.write_all(contents.as_bytes())?;

        Ok(())
    }

    // Converts a blog post to a short excerpt string
    fn blog_to_blurb(&self, b: &Blog) -> Result<String> {
        // Get the layout for the blurb
        let layout = self.repo.get_layout("blurb.layout.html")?;
        replace_placeholders(&layout.html, hashmap! { "title" => b.title.as_str() })
    }
}

fn replace_placeholders(layout: &str, args: HashMap<&str, &str>) -> Result<String> {
    let mut result = layout.to_owned();

    // Cause we need to do this in reverse order...
    let mut v = Vec::new();

    // Can use a lazy macro to make this static
    let re = Regex::new(r"\{\{([a-z.]+)\}\}").unwrap();
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
