use cli_twitter::{
    commands::parse_command,
    data::{tasks::TaskDao, DbPool},
    domain::TaskStatus,
};

#[test]
fn parse_tasks_variants() {
    let cmd = parse_command("/tasks ls open").unwrap();
    match cmd {
        cli_twitter::commands::Command::TasksList { status } => {
            assert_eq!(status, Some(TaskStatus::Open));
        }
        _ => panic!("wrong command"),
    }
}

#[test]
fn dao_add_and_complete() {
    let pool = DbPool::in_memory().unwrap();
    let conn = pool.connection();
    let dao = TaskDao::new(&conn);
    let task = dao.add_task("demo", "demo desc").unwrap();
    assert_eq!(task.status, TaskStatus::Open);
    let listed = dao.list_tasks(None).unwrap();
    assert_eq!(listed.len(), 1);
    let done = dao.mark_done(task.id).unwrap().unwrap();
    assert_eq!(done.status, TaskStatus::Done);
}
