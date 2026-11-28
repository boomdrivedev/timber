//! timber's end-to-end set in the fast layer, in one binary: the tests
//! that run the binary, git or the served page against scratch
//! repositories under the sealed scratch home (rule
//! logic-is-tested-as-units-and-few-tests-run-the-binary). The tools
//! layer's tests and the hosted ones keep binaries of their own, since
//! each sets a variable of its process, or its working directory, that a
//! test beside it would read.

#[path = "../scratch/mod.rs"]
mod scratch;

mod build_stamp;
mod cli;
mod herdr_documents;
mod host;
mod landing_branch;
mod ledger_git_ref;
mod policy_file;
mod promote;
mod prune;
mod serve;
mod status_across_machines;
