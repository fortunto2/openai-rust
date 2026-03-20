// Beta resources — Assistants, Threads, Runs, Vector Stores
// All beta endpoints require the OpenAI-Beta: assistants=v2 header.

pub mod assistants;
pub mod runs;
pub mod threads;
pub mod vector_stores;

pub(crate) const BETA_HEADER: (&str, &str) = ("OpenAI-Beta", "assistants=v2");
