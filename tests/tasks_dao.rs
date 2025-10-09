use cli_twitter::data::{tasks::TaskDao, AppDatabase, DatabaseConfig};
use cli_twitter::domain::TaskStatus;

fn temp_db() -> AppDatabase {
    let path = std::env::temp_dir().join(format!("cli-twitter-test-{}.db", uuid::Uuid::new_v4()));
    let cfg = DatabaseConfig { path };
    AppDatabase::open(&cfg).unwrap()
}

#[test]
fn create_and_complete_task() {
    let db = temp_db();
    let dao = TaskDao::new(&db);
    let task = dao.add("Test", "Desc").unwrap();
    assert_eq!(task.status, TaskStatus::Open);
    let done = dao.mark_done(task.id).unwrap().unwrap();
    assert_eq!(done.status, TaskStatus::Done);
}
