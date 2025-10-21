use anyhow::{Context, Result};
use std::fs::OpenOptions;
use std::io::Write;
use time::OffsetDateTime;

use crate::data::cache::CacheDao;
use crate::data::AppDatabase;
use crate::domain::{Report, Task};

pub fn generate_report(db: &AppDatabase, task: &Task) -> Result<Report> {
    let now = OffsetDateTime::now_utc();
    std::fs::create_dir_all("reports").context("creating reports directory")?;
    let filename = format!(
        "reports/{}_task-{}.md",
        now.format(&time::macros::format_description!("%Y%m%d_%H%M"))?,
        task.id
    );
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&filename)
        .with_context(|| format!("creating report {}", filename))?;

    let cache = CacheDao::new(db);
    let posts = cache.list_posts(50)?;
    let summary = format!(
        "Task #{id} \nStatus: {status}\nTitle: {title}\nDescription: {desc}\nCompleted at: {done:?}\nCached posts: {count}\n",
        id = task.id,
        status = task.status.as_str(),
        title = task.title,
        desc = task.description,
        done = task.done_at,
        count = posts.len()
    );
    writeln!(file, "# Task Report {}", task.id)?;
    writeln!(file, "\n{}", summary)?;
    writeln!(file, "## Recent Cached Posts")?;
    for post in posts.iter().take(10) {
        writeln!(
            file,
            "- [{}] {} ({})",
            post.audience,
            post.body.replace('\n', " "),
            post.author_handle
                .clone()
                .unwrap_or_else(|| post.author.to_string())
        )?;
    }

    Ok(Report {
        id: 0,
        task_id: Some(task.id),
        path: filename,
        summary,
        created_at: now,
    })
}
