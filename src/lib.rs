//! Simple in-memory <KV> store library, made as part of #100DaysOfCode challenge
//! following the [ping-cap talent plant](https://github.com/pingcap/talent-plan/blob/master/courses/rust/projects/project-1/README.md)
//#![deny(missing_docs)]
/*
 * A good structure for documentation (used in stdl) is:
   - [short explanation of what item does]\n
   - [code example showing how to use it]\n
   - [Optional: more expalantions and code examples in case some specific cases are not straightforward]
ref: https://blog.guillaume-gomez.fr/articles/2020-03-12+Guide+on+how+to+write+documentation+for+a+Rust+crate
guideline: https://rust-lang.github.io/api-guidelines/documentation.html
*/
pub use error::{ErrorKind, KvsError, Result};
pub use kv::KvStore;
mod error;
mod kv;
