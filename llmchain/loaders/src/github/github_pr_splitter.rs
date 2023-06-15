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

use anyhow::Result;
use glob::Pattern;
use log::info;
use patch::Patch;

use crate::text::TextSplitter;
use crate::Document;
use crate::DocumentSplitter;

pub struct GithubPRDiffSplitter {
    pub splitter_chunk_size: usize,
    skips: Vec<String>,
}

impl GithubPRDiffSplitter {
    pub fn create() -> Self {
        GithubPRDiffSplitter {
            splitter_chunk_size: 2000,
            skips: vec![],
        }
    }

    pub fn with_chunk_size(mut self, chunk_size: usize) -> Self {
        self.splitter_chunk_size = chunk_size;
        self
    }

    pub fn with_skips(mut self, skips: Vec<String>) -> Self {
        self.skips = skips;
        self
    }
}

impl DocumentSplitter for GithubPRDiffSplitter {
    fn separators(&self) -> Vec<String> {
        vec![]
    }

    fn split_documents(&self, documents: &[Document]) -> Result<Vec<Document>> {
        // To diff documents by files.
        let mut diff_documents = vec![];
        for document in documents {
            let content = Box::leak(document.content.clone().into_boxed_str());
            let patches = Patch::from_multiple(content)?;
            for patch in patches {
                let mut need_skip = false;
                for skip in &self.skips {
                    let pattern = Pattern::new(skip)?;
                    if pattern.matches(&patch.old.path) || pattern.matches(&patch.new.path) {
                        info!("Skip diff file: old:{}, new:{}", patch.old, patch.new);
                        need_skip = true;
                        break;
                    }
                }

                if !need_skip {
                    let content = format!("{}", patch);
                    diff_documents.push(Document::create(&document.path, &content));
                }
            }
        }
        info!(
            "Split {} documents into {} diff documents",
            documents.len(),
            diff_documents.len()
        );

        let text_splitter = TextSplitter::create()
            .with_chunk_size(self.splitter_chunk_size)
            .with_separators(self.separators());
        let result = text_splitter.split_documents(&diff_documents)?;
        info!(
            "Split {} diff documents into {} text documents",
            diff_documents.len(),
            result.len()
        );

        Ok(result)
    }
}
