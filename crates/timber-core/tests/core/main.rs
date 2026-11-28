//! The core's tests over the fakes, in one binary: each seam a fake, the
//! history a scratch repository in git, which the core reads and pushes
//! to with git (rule logic-is-tested-as-units-and-few-tests-run-the-binary).

mod common;
mod world;

mod after;
mod apply;
mod cache;
mod carry;
mod claim;
mod declared;
mod follow;
mod gate;
mod history;
mod in_force;
mod landed;
mod ledger;
mod notes;
mod prune;
mod publish;
mod push;
mod record;
mod render;
mod status;
mod verify;
mod version;
