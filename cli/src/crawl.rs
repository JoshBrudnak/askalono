// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License").
// You may not use this file except in compliance with the License.
// A copy of the License is located at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// or in the "license" file accompanying this file. This file is distributed
// on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
// express or implied. See the License for the specific language governing
// permissions and limitations under the License.

use std::fs::read_to_string;
use std::path::Path;

use failure::Error;

use askalono::TextData;

use super::identify::identify_data;
use super::util::*;

pub fn crawl(
    cache_filename: &Path,
    directory: &Path,
    follow_links: bool,
    glob: Option<&str>,
) -> Result<(), Error> {
    use std::sync::Arc;
    use ignore::types::TypesBuilder;
    use ignore::{WalkBuilder, WalkState};

    let store = load_store(cache_filename)?;
    let lock = Arc::new(store);

    let mut types_builder = TypesBuilder::new();
    if let Some(globstr) = glob {
        types_builder.add("custom", globstr)?;
        types_builder.select("custom");
    } else {
        types_builder.add_defaults();
        types_builder.select("license");
    }
    let matcher = types_builder.build().unwrap();

    WalkBuilder::new(directory)
        .types(matcher)
        .follow_links(follow_links)
        .build_parallel()
        .run(|| {
            let local_store = lock.clone();
            Box::new(move |result| {
                if !result.is_ok() {
                    eprintln!("{}", result.unwrap_err());
                    return WalkState::Skip;
                }

                let entry = result.unwrap();
                let metadata = entry.metadata();
                if !metadata.is_ok() {
                    eprintln!("{}", metadata.unwrap_err());
                    return WalkState::Skip;
                }

                if metadata.unwrap().is_dir() {
                    return WalkState::Continue;
                }

                let path = entry.path();

                if let Ok(content) = read_to_string(path) {
                    let data = TextData::new(&content);
                    match identify_data(&local_store, &data, false, false) {
                        Ok(res) => {
                            print!("{}\n{}", path.display(), res);
                        },
                        Err(err) => {
                            eprintln!("{}\nError: {}", path.display(), err);
                        },
                    };
                }

                WalkState::Continue
            })
        });

    Ok(())
}
