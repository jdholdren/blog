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

// A step of making the site:
// Gets access to a list of blogs and the renderer.
type GenerationStep = dyn FnOnce(&Vec<Blog>, &layout::Renderer) -> Vec<String>;

pub fn generate_all(mut blogs: Vec<Blog>, renderer: layout::Renderer) {
    blogs.sort_by(|a, b| b.publish_date.partial_cmp(&a.publish_date).unwrap());
    // The pages we've built to put into the sitemap at the end
    let mut page_list: Vec<String> = vec![];
    let steps: Vec<Box<GenerationStep>> =
        vec![Box::new(generate_index), Box::new(generate_all_posts)];

    for step in steps {
        let made_pages = step(&blogs, &renderer);
        for page in made_pages {
            page_list.push(page)
        }
    }

    // Cap it off with the sitelist
    generate_sitemap(page_list)
}

fn generate_index(blogs: &Vec<Blog>, r: &layout::Renderer) -> Vec<String> {
    // The index displays the 3 most recent blogs.
    let blogs = &blogs[0..3];

    let mut blogs_arg = String::new();
    for blog in blogs {
        let blurb = blog_to_blurb(blog, r);
        blogs_arg.push_str(&blurb);
    }

    let mut args: HashMap<&str, String> = HashMap::new();
    args.insert("latest_posts", blogs_arg);

    let contents = r.render_layout("index", &args).unwrap();
    let mut f = File::create("./generated/index.html").unwrap();
    f.write_all(contents.as_bytes()).unwrap();

    vec![String::from("https://jamesholdren.com")]
}

fn generate_all_posts(blogs: &Vec<Blog>, r: &layout::Renderer) -> Vec<String> {
    let mut generated_pages = vec![];

    let mut blogs_arg = String::new();
    for blog in blogs {
        let blurb = blog_to_blurb(blog, r);
        blogs_arg.push_str(&blurb);
    }

    let contents = r
        .render_layout(
            "all_posts",
            &hashmap! {
                "posts" => blogs_arg,
                "posts_selected" => String::from("true"),
            },
        )
        .expect("rendering all posts layout");
    std::fs::create_dir_all("./generated/posts").expect("creating posts directory");
    let mut f = File::create("./generated/posts/index.html").expect("creating /posts/index.html");
    f.write_all(contents.as_bytes())
        .expect("writing bytes to /posts/index.html");
    generated_pages.push("https://jamesholdren.com/posts/".to_string());

    // For each blog post, generate its page
    for blog in blogs.iter().filter(|blog| blog.external.is_none()) {
        let contents = r
            .render_layout(
                "post",
                &hashmap! {
                    "title" => blog.title.to_owned(),
                    "description" => blog.excerpt.to_owned(),
                    "publish_date" => blog.display_date.to_owned(),
                    "contents" => blog.html.to_owned(),
                },
            )
            .unwrap();
        std::fs::create_dir_all(format!("./generated/posts/{}/", blog.slug)).unwrap();
        let mut f = File::create(format!("./generated/posts/{}/index.html", blog.slug)).unwrap();
        f.write_all(contents.as_bytes()).unwrap();

        // Add the post to the sitemap
        generated_pages.push(format!("https://jamesholdren.com/posts/{}/", blog.slug));
    }

    generated_pages
}

// Converts a blog post to a short excerpt string
fn blog_to_blurb(b: &Blog, r: &layout::Renderer) -> String {
    // Use the external link or slug it out
    let link = b
        .external
        .to_owned()
        .unwrap_or(format!("/posts/{}/", b.slug));

    r.render_layout(
        "blurb",
        &hashmap! {
            "title" => b.title.to_owned(),
            "excerpt" => b.excerpt.to_owned(),
            "publish_date" => b.display_date.to_owned(),
            "link" => link,
        },
    )
    .unwrap()
}

// Takes the page_list and turns it into a text sitemap
fn generate_sitemap(page_list: Vec<String>) {
    let mut f = File::create("./generated/sitemap.txt").unwrap();
    for page in &page_list {
        writeln!(f, "{page}").unwrap();
    }
}
