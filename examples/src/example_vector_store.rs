// Copyright 2023 Shafish Labs.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::sync::Arc;

use anyhow::Result;
use env_logger::Env;
use llmchain_embeddings::OpenAIEmbedding;
use llmchain_sources::Document;
use llmchain_vector_stores::DatabendVectorStore;
use llmchain_vector_stores::VectorStore;
use log::info;

/// EXPORT OPENAI_API_KEY=<your-openai-api-key>
/// EXPORT DATABEND_DSN=<your-databend-dsn>
/// cargo run --bin example_vector_store
#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| {
            "OPENAI_API_KEY is empty, please EXPORT OPENAI_API_KEY=<your-openai-api-key>"
                .to_string()
        })
        .unwrap();
    let dsn = std::env::var("DATABEND_DSN")
        .map_err(|_| {
            "DATABEND_DSN is empty, please EXPORT DATABEND_DSN=<your-databend-dsn>".to_string()
        })
        .unwrap();

    // Sample documents.
    let documents = vec![
        Document::create("1.md", "hello"),
        Document::create("2.md", "llmchain.rs"),
    ];

    // create openai embedding.
    let openai_embedding = Arc::new(OpenAIEmbedding::create(&api_key));

    // create databend vector store.
    let databend = DatabendVectorStore::create(&dsn, openai_embedding);
    databend.init().await?;

    // add documents to vector store.
    let uuids = databend.add_documents(documents).await?;
    info!("embedding uuids:{:?}", uuids);

    // query a similarity document.
    let query = "llmchain";
    let similarities = databend.similarity_search("llmchain", 1).await?;
    info!("query:{}, similarity documents:{:?}", query, similarities);

    Ok(())
}
