use sourcemap::SourceMap;
use std::collections::HashMap;

pub fn unpack_sources(sm: &SourceMap) -> HashMap<String, String> {
  let mut map = HashMap::new();
  for (idx, source) in sm.sources().enumerate() {
    if let Some(view) = sm.get_source_view(idx as u32) {
      map.insert(source.to_string(), view.source().to_string());
    }
  }
  map
}
