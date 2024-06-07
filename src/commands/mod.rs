mod add;
mod apply;
mod clean;
mod exec;
mod get;
mod init;
mod install;
mod list;
mod remove;
mod run;
mod uninstall;
mod upgrade;
mod watch;

pub use self::add::*;
pub use self::apply::*;
pub use self::clean::*;
pub use self::exec::*;
pub use self::get::*;
pub use self::init::*;
pub use self::install::*;
pub use self::list::*;
pub use self::remove::*;
pub use self::run::*;
pub use self::uninstall::*;
pub use self::upgrade::*;
pub use self::watch::*;

use anyhow::Result;
use enum_dispatch::enum_dispatch;

#[enum_dispatch]
pub trait Command {
    fn dispatch(&self) -> Result<()>;
    fn error_context(&self) -> String;
}
