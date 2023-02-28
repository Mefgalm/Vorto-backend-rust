use std::{
    collections::HashMap,
    hash::Hash,
};

use indexmap::IndexMap;

use crate::error::{VortoError, VortoErrorCode, VortoResult};

pub fn group<T, G, K, V>(
    datas: &Vec<T>,
    group_fn: impl Fn(&T) -> &G,
    key_fn: impl Fn(&T) -> K,
    value_fn: impl Fn(&T) -> Option<V>,
) -> Vec<(K, Vec<V>)>
where
    G: Eq + Copy + Hash,
    K: Clone,
    V: Clone,
{
    let mut map: IndexMap<G, (K, Vec<V>)> = IndexMap::new();
    for d in datas {
        let group_key = group_fn(d);
        let value_opt = value_fn(d);
        if let Some((_, vec)) = map.get_mut(group_key) {
            if let Some(value) = value_opt {
                vec.push(value);
            }
        } else {
            let new_vec = if let Some(value) = value_opt {
                vec![value]
            } else {
                vec![]
            };
            map.insert(group_key.clone(), (key_fn(d), new_vec));
        }
    }
    map.into_iter().map(|x| x.1).collect()
}

pub fn vec_to_map<T, K, V>(
    vec: &Vec<T>,
    key_fn: impl Fn(&T) -> K,
    value_fn: impl Fn(&T) -> V,
) -> HashMap<K, V>
where
    K: Hash + Eq,
{
    let mut hash_map: HashMap<K, V> = HashMap::new();
    for v in vec {
        hash_map.insert(key_fn(&v), value_fn(&v));
    }
    hash_map
}

pub fn reduce_results<T: Clone>(results: &Vec<VortoResult<T>>) -> VortoResult<Vec<T>> {
    let mut oks = vec![];
    let mut errors = vec![];
    for result in results {
        match result {
            VortoResult::Ok(r) => oks.push(r.clone()),
            VortoResult::Err(e) => errors.push(e.clone()),
        }
    }
    if errors.is_empty() {
        VortoResult::Ok(oks)
    } else {
        let message = errors
            .iter()
            .map(|e| format!("code: {} message: {}", e.code, e.message))
            .collect::<Vec<String>>()
            .join(", ");
        VortoResult::Err(VortoError::new(VortoErrorCode::Validation, message))
    }
}
