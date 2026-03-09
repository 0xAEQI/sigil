//! Persistent memory with full-text search and vector similarity.
//!
//! Combines SQLite FTS5 keyword search ([`SqliteMemory`]) with vector embeddings
//! ([`VectorStore`]) using Reciprocal Rank Fusion and MMR reranking ([`hybrid`]).
//! Text chunking ([`chunker`]) splits documents into overlapping segments for indexing.
//!
//! Used by agent workers for long-term memory recall during task execution.

pub mod chunker;
pub mod hybrid;
pub mod sqlite;
pub mod vector;

pub use chunker::{chunk_default, chunk_text, Chunk};
pub use hybrid::{merge_scores, mmr_rerank, ScoredResult};
pub use sqlite::SqliteMemory;
pub use vector::{cosine_similarity, VectorStore};
