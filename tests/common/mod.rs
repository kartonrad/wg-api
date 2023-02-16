use std::io::{BufReader, BufRead};

use std::process::{ Child, ChildStdout, Stdio};

use futures_util::lock::Mutex;
use sqlx::{Pool, Postgres};
use sqlx::postgres::PgPoolOptions;

use async_once::AsyncOnce;
use lazy_static::{lazy_static};

/* 
#[derive(Clone)]
pub struct GlobalTestStateHandle {
    pub inner: Arc<GlobalTestState>
}
impl Deref for GlobalTestStateHandle {
    type Target = Arc<GlobalTestState>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl DerefMut for GlobalTestStateHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
impl Drop for GlobalTestStateHandle {
    fn drop(&mut self) {
        if Arc::strong_count(&self.inner) <= 2 {
            let _ = self.child.get_mut().kill().expect("To be able to drop thing");
            self.child.get_mut().wait().expect("Wait");
            panic!("LAST TEST!")
        }
    }
}*/

pub struct GlobalTestState {
    pub child: Mutex<Child>,
    pub stdout: Mutex<BufReader<ChildStdout>>,
    pub pool: Pool<Postgres>
}

lazy_static! {
    pub static ref GLOBAL_TEST_STATE: AsyncOnce<GlobalTestState> = AsyncOnce::new(async{
        init_tests().await
    });
}

pub async fn init_tests() -> GlobalTestState {
    println!("Initializing Database Pool...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://web:heretoserve@localhost/wg-db").await.expect("Connection success");

    // SEEDING LOGIC, get db into state

    println!("LOG");

    let mut child = std::process::Command::new(".\\target\\debug\\wg-api")
        //.arg("run").arg("--quiet")
        //.arg("--release")
        .current_dir(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .env("NO_DOTENV", "e")
        .env("RUST_LOG", "trace,actix=info,mio=info,sqlx=info")
        .env("HOST", "localhost")
        .env("PORT", "4269")
        .env("DATABASE_URL", "postgres://web:heretoserve@localhost/wg-db")
        .env("JWT_SECRET", "bears")
        .stdout(Stdio::piped())
        .spawn().expect("Initialisation to succeed.");

    let mut stdout = BufReader::new( child.stdout.take().unwrap() );

    // while still running
    let mut line = String::new();

    let mut ready = false;
    while child.try_wait().expect("Waiting to work").is_none() {
        stdout.read_line(&mut line).expect("Valid utf8");

        //println!("LINE {}", line);

        if line.trim() == "#READY!localhost:4269" {
            println!("Child Server started!");
            ready = true;
            break;
        }
        line.clear();
    }

    if !ready {
        panic!("Process failed with exit code: {}", child.wait().expect("ExitCode"));
    } /* else {
        panic!("Child  Server started!");
    } */

    GlobalTestState {
        pool,
        child: Mutex::new(child),
        stdout: Mutex::new(stdout)
    }
}