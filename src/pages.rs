use crate::repo;

use crate::Error;
use repo::Repo;

pub fn generate_index(repo: &Repo) -> Result<(), Error> {
    let blogs = repo.all_blogs()?;

    Ok(())
}
