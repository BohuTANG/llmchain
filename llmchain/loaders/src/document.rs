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

#[derive(Debug, Clone)]
pub struct DocumentSettings {
    pub splitter_chunk_size: usize,
}

#[derive(Debug, Clone)]
pub struct DocumentMeta {
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct Document {
    pub meta: DocumentMeta,
    pub content: String,
}

#[async_trait::async_trait]
pub trait DocumentLoader {
    async fn load(&self, path: &str) -> Result<Vec<Document>>;
}
