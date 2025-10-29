use std::collections::{
    HashMap,
    HashSet,
};

use dashmap::{
    DashMap,
    DashSet,
};
use rustc_hash::FxBuildHasher;

pub type FxDashMap<K, V> = DashMap<K, V, FxBuildHasher>;
pub type FxDashSet<K> = DashSet<K, FxBuildHasher>;
pub type FxHashMap<K, V> = HashMap<K, V, FxBuildHasher>;
pub type FxHashSet<K> = HashSet<K, FxBuildHasher>;
