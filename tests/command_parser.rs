use cli_twitter::commands::{self, Command};
use cli_twitter::domain::AudienceScope;

#[test]
fn parse_post_command() {
    let cmd = commands::parse_command("/post \"hello\" audience:public").unwrap();
    match cmd {
        Command::Post { text, audience } => {
            assert_eq!(text, "hello");
            assert_eq!(audience, AudienceScope::Public);
        }
        _ => panic!("unexpected command"),
    }
}

#[test]
fn parse_login_command() {
    let cmd = commands::parse_command("/login email:test@example.com pw:secret").unwrap();
    match cmd {
        Command::Login { email, password } => {
            assert_eq!(email, "test@example.com");
            assert_eq!(password, "secret");
        }
        _ => panic!("unexpected command"),
    }
}
