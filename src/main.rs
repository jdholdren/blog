use anyhow::{bail, Context, Result};
use pulldown_cmark::CodeBlockKind;
use pulldown_cmark::CowStr;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use syntect::highlighting::Theme;

use fs_extra::dir::CopyOptions;
use pulldown_cmark::html;
use pulldown_cmark::Event;
use pulldown_cmark::Parser;
use pulldown_cmark::Tag;

// Syntax highlighting deps
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

mod pages;

fn main() -> Result<()> {
    // Blow away the destination folder
    fs::remove_dir_all("./generated").ok();

    // Make sure we have a folder to store the generated blog
    fs::create_dir("./generated").expect("error creating generated dir");
    fs::create_dir("./generated/static").expect("error creating static dir");

    // Copies the static directory over to the generated folder
    let options = CopyOptions::new();
    fs_extra::dir::copy("./static", "./generated", &options)
        .expect("could not copy static assets over");

    let ls = layouts()?;
    let posts = posts()?;

    let mut p = pages::Pages::new(posts, pages::Renderer::new(ls));

    // Pages to be generated
    p.generate_index()?;
    p.generate_all_posts()?;
    p.generate_sitemap()?;

    Ok(())
}

const LAYOUT_DIR: &str = "./layouts/";
const LAYOUT_SUFFIX: &str = ".layout.html";

fn layouts() -> Result<HashMap<String, String>> {
    let mut m: HashMap<String, String> = HashMap::new();

    for mut layout in walk_directory(LAYOUT_DIR)? {
        let contents = fs::read_to_string(&layout)?;

        // Trim off the folder name (and suffix) to get the layout id
        layout.replace_range(0..LAYOUT_DIR.len(), "");
        layout = layout.replace(LAYOUT_SUFFIX, "");

        // Insert it into the db
        m.insert(layout, contents);
    }

    Ok(m)
}

fn posts() -> Result<Vec<pages::Blog>> {
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["base16-eighties.dark"];

    walk_directory("./posts")?
        .into_iter()
        .map(|blog_name| {
            let contents = fs::read_to_string(&blog_name)?;
            render_blog(blog_name, contents, &ss, theme)
        })
        .collect()
}

// Performs all the logic for parsing a blog post into its HTML
fn render_blog(
    blog_name: String,
    contents: String,
    ss: &SyntaxSet,
    theme: &Theme,
) -> Result<pages::Blog> {
    let mut parser = pulldown_cmark::Parser::new_ext(&contents, pulldown_cmark::Options::all());

    // Meta information about the blog
    let front_matter = parse_frontmatter(&mut parser)
        .with_context(|| format!("could not parse frontmatter for {blog_name}"))?;
    let metadata = frontmatter_to_meta(&front_matter);

    // This stores events we've already gone through
    let mut events = Vec::new();
    // The string we need to highlight
    let mut to_highlight = String::new();
    // Loop through and handle specifically entering/exiting and being with a code block
    let mut in_code_block = false;
    let mut syntax = ss.find_syntax_plain_text();
    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code_block = true;

                // Determine the syntax set from the information in the start event
                if let CodeBlockKind::Fenced(language) = kind {
                    syntax = ss.find_syntax_by_extension(&language).unwrap();
                }
            }
            Event::Text(t) => {
                if in_code_block {
                    // If we're in a code block, build up the string of text
                    to_highlight.push_str(&t);
                    continue;
                }
                events.push(Event::Text(t)) // Just forward the event through
            }
            Event::End(Tag::CodeBlock(_)) => {
                if !in_code_block {
                    continue;
                }

                let html = highlighted_html_for_string(&to_highlight, &ss, &syntax, &theme)?;
                // I'm missing something here for sure, but CowStr does implement From Cow<str>
                let c = Into::<CowStr>::into(Cow::<str>::Owned(html));
                events.push(Event::Html(c));
                to_highlight = String::new();
                in_code_block = false;
            }
            e => {
                events.push(e);
            }
        }
    }

    let mut html_str = String::new();
    html::push_html(&mut html_str, events.into_iter());

    // Insert it into the db
    Ok(pages::Blog {
        id: blog_name,
        title: metadata.title,
        publish_date: metadata.publish_date,
        excerpt: metadata.excerpt,
        html: html_str,
        slug: metadata.slug,
        external: metadata.external,
    })
}

// Produces all files within a directory
fn walk_directory(dir: &str) -> Result<Vec<String>> {
    let mut files: Vec<String> = vec![];

    for entry in fs::read_dir(dir).context("error reading dir")? {
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
fn parse_frontmatter(parser: &mut Parser) -> Result<Frontmatter> {
    let mut reading_frontmatter = false;
    let mut fm = Frontmatter::new();
    let mut building = String::new(); // If we're building a text value

    'events: for event in parser {
        if !reading_frontmatter {
            match event {
                pulldown_cmark::Event::Rule => {
                    reading_frontmatter = true; // start parsing
                    continue;
                }
                pulldown_cmark::Event::SoftBreak => {}
                _ => {
                    bail!("frontmatter not found");
                }
            }
        }

        let mut should_parse = false;
        if let pulldown_cmark::Event::SoftBreak = event {
            should_parse = true;
        } else if let pulldown_cmark::Event::End(_) = event {
            should_parse = true;
        }

        if should_parse {
            // The first delimeter is a ':', then everything after is the value
            let pos = building.chars().position(|c| c == ':').unwrap();
            let key = building[..pos].to_string();
            let value = building[(pos + 1)..].trim().to_string();

            // New we're good to insert into our fm
            fm.insert(key, value);

            // Clear the built variable
            building = String::new();

            continue;
        } else if let pulldown_cmark::Event::Text(tag) = event {
            building.push_str(&tag);
            continue;
        } else if let pulldown_cmark::Event::Rule = event {
            break 'events;
        }
    }

    Ok(fm)
}

struct Meta {
    title: String,
    publish_date: String,
    excerpt: String,
    slug: String,
    external: Option<String>,
}

fn frontmatter_to_meta(fm: &Frontmatter) -> Meta {
    Meta {
        title: fm.get("title").unwrap().to_string(),
        publish_date: fm.get("publishDate").unwrap().to_string(),
        excerpt: fm.get("excerpt").unwrap().to_owned(),
        slug: fm.get("slug").unwrap_or(&String::new()).to_owned(),
        external: fm.get("external").map(|f| f.to_owned()),
    }
}
