use crate::err_context::ErrContext;

pub fn push_all(
    repo: &mut git2::Repository,
) -> Result<(), crate::err_context::ErrWithContext<git2::Error>> {
    let mut index = repo.index().context("Get index")?;
    index
        .add_all(&["*"], git2::IndexAddOption::DEFAULT, None)
        .context("Add path")?;
    index.write().context("Write index")?;

    let sig = repo.signature().context("Get repo signature")?;
    repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        "",
        &repo
            .find_tree(index.write_tree().context("Write index tree")?)
            .context("Find index tree")?,
        &[&repo
            .find_commit(
                repo.head()
                    .context("Get HEAD")?
                    .target()
                    .expect("No HEAD ref in repo"),
            )
            .context("Get HEAD commit")?],
    )
    .context("Commit")?;

    // FIXME: use push_update_reference callback, as recommended
    repo.find_remote("origin")
        .context("Get remote origin")?
        .push(&["refs/heads/master"], None)
        .context("Push to origin")?;

    Ok(())
}
