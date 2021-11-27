use crate::err_context::ErrContext;

pub fn push_existing(
    repo: &git2::Repository,
) -> Result<(), crate::err_context::ErrWithContext<git2::Error>> {
    let master = repo
        .find_branch("master", git2::BranchType::Remote)
        .context("Local master branch")?
        .into_reference()
        .target()
        .map(|oid| repo.find_commit(oid))
        .transpose()
        .context("Find commit for master")?;

    // FIXME: that's rather strange to see, but otherwise we run into lifetime problems
    let parents: Box<[&_]> = match master {
        Some(ref commit) => [commit].into(),
        None => [].into(),
    };

    commit_and_push(repo, &*parents)
}

pub fn commit_and_push(
    repo: &git2::Repository,
    parents: &[&git2::Commit],
) -> Result<(), crate::err_context::ErrWithContext<git2::Error>> {
    let mut origin = repo.find_remote("origin").context("Get remote origin")?;

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
        parents,
    )
    .context("Commit")?;

    origin
        .push(&["refs/heads/master"], None)
        .context("Push to origin")?;

    Ok(())
}
