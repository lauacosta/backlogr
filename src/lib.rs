pub mod cli;
pub mod integrations;

pub trait ExitOnError<T> {
    fn or_exit(self) -> T;
}
