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

use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use env_logger::Env;
use llmchain_embeddings::OpenAIEmbedding;
use llmchain_llms::OpenAI;
use llmchain_llms::LLM;
use llmchain_prompts::DocumentRetrievalPrompt;
use llmchain_prompts::Prompt;
use llmchain_sources::DirectoryLoader;
use llmchain_sources::DocumentLoader;
use llmchain_sources::DocumentSplitter;
use llmchain_sources::LocalDisk;
use llmchain_sources::MarkdownLoader;
use llmchain_sources::MarkdownSplitter;
use llmchain_vector_stores::DatabendVectorStore;
use llmchain_vector_stores::VectorStore;
use log::info;

/// EXPORT OPENAI_API_KEY=<your-openai-api-key>
/// EXPORT DATABEND_DSN=<your-databend-dsn>
/// cargo run --bin example_document_qa <embedding|query>
#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Get the key.
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

    let args: Vec<String> = env::args().collect();
    if !args.is_empty() {
        let arg = args.get(1).unwrap();
        match arg.as_str() {
            "embedding" => embeddings(&api_key, &dsn).await?,
            "query" => query(&api_key, &dsn).await?,
            _ => {
                info!("cargo run --bin example_document_qa [embedding|query]")
            }
        }
    }

    Ok(())
}

async fn embeddings(api_key: &str, databend_dsn: &str) -> Result<()> {
    // dir.
    let curdir = std::env::current_dir()?.to_str().unwrap().to_string();
    let testdata_dir = format!("{}/examples/testdata", curdir);
    let directory_dir = format!("{}/markdowns/", testdata_dir);

    // Embedding.
    {
        let start = Instant::now();
        // Loader.
        info!("Prepare to load all the documents {}", directory_dir);
        let directory_loader = DirectoryLoader::create(LocalDisk::create()?)
            .with_loader("**/*.md", MarkdownLoader::create(LocalDisk::create()?));
        let documents = directory_loader.load(&directory_dir)?;
        info!(
            "Load all the documents {} done, cost: {}",
            directory_dir,
            start.elapsed().as_secs()
        );

        // Splitter.
        info!(
            "Prepare to split all the documents, count: {}",
            documents.len()
        );
        let start = Instant::now();
        let documents = MarkdownSplitter::create().split_documents(&documents)?;
        info!(
            "Split all to documents, count: {}, cost: {}",
            documents.len(),
            start.elapsed().as_secs()
        );

        // embedding.
        info!(
            "Prepare to indexing the documents, count: {}",
            documents.len()
        );
        let start = Instant::now();
        let openai_embedding = Arc::new(OpenAIEmbedding::create(api_key));
        let databend = DatabendVectorStore::create(databend_dsn, openai_embedding);
        databend.init().await?;

        // indexing.
        let uuids = databend.add_documents(documents).await?;
        info!(
            "Indexing the documents done, count: {}, cost: {}",
            uuids.len(),
            start.elapsed().as_secs()
        );

        Ok(())
    }
}

async fn query(api_key: &str, databend_dsn: &str) -> Result<()> {
    let start = Instant::now();
    let question = "how to do COPY in databend";

    let openai_embedding = Arc::new(OpenAIEmbedding::create(api_key));
    let databend = DatabendVectorStore::create(databend_dsn, openai_embedding);
    databend.init().await?;
    let similarities = databend.similarity_search(question, 3).await?;
    info!(
        "query: {}, similarity documents: {:?}, cost: {}",
        question,
        similarities.len(),
        start.elapsed().as_secs()
    );

    let contexts = similarities
        .iter()
        .map(|x| format!("context:{}\nsource:{}", x.path, x.content))
        .collect::<Vec<_>>()
        .join("");
    let prompt_template = DocumentRetrievalPrompt::create().with_instructions(vec!["Present your answer in markdown format, including code snippets if have, format the code snippets with SQL type if necessary.",
                                                                                   "Do not include any links or external references in your response.\n",
                                                                                   "Do not change the code snippets.\n",
                                                                                   "Do not change the SQL syntax, please don't make up the function.\n",
                                                                                   "Do not change explain any code snippets.\n",
                                                                                   "Make the whole answer as short as possible to keep the code snippets.\n"
    ]);
    let mut input_variables = HashMap::new();
    input_variables.insert("question", question);
    input_variables.insert("contexts", &contexts);
    let prompt = prompt_template.format(input_variables)?;

    //
    let openai_llm = OpenAI::create(api_key);
    let answer = openai_llm.generate(&prompt).await?;
    info!("question: {}", question);
    info!("answer: {:?}", answer);
    Ok(())
}
