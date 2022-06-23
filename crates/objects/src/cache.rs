use std::{ops::Deref, path::PathBuf, sync::Arc};

use bevy::{
    asset::{Asset, AssetPath, LoadState},
    prelude::*,
};
use de_core::{
    objects::{ActiveObjectType, InactiveObjectType, ObjectType},
    state::GameState,
};
use enum_map::{enum_map, EnumMap};
use iyes_loopless::prelude::*;
use iyes_progress::prelude::*;

use crate::{
    ichnography::Ichnography,
    loader::{Footprint, ObjectLoader},
};

pub(crate) struct CachePlugin;

impl Plugin for CachePlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<Footprint>()
            .add_asset_loader(ObjectLoader)
            .add_enter_system(GameState::Loading, setup)
            .add_system(
                check_status
                    .track_progress()
                    .run_in_state(GameState::Loading),
            );
    }
}

#[derive(Clone)]
pub struct ObjectCache {
    inner: Arc<InnerCache>,
}

impl ObjectCache {
    fn new(inner: InnerCache) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }
}

impl Deref for ObjectCache {
    type Target = InnerCache;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}

pub struct InnerCache {
    objects: EnumMap<ObjectType, CacheItem>,
}

impl InnerCache {
    pub(crate) fn get(&self, object_type: ObjectType) -> &CacheItem {
        &self.objects[object_type]
    }
}

pub(crate) struct CacheItem {
    scene: Handle<Scene>,
    ichnography: Ichnography,
}

impl CacheItem {
    pub(crate) fn scene(&self) -> Handle<Scene> {
        self.scene.clone()
    }

    pub(crate) fn ichnography(&self) -> &Ichnography {
        &self.ichnography
    }
}

struct CacheLoader {
    objects: EnumMap<ObjectType, ItemLoader>,
}

impl CacheLoader {
    fn load(server: &AssetServer) -> Self {
        Self {
            objects: enum_map! {
                ObjectType::Active(ActiveObjectType::Base) => ItemLoader::from_name(server, "base"),
                ObjectType::Active(ActiveObjectType::PowerHub) => ItemLoader::from_name(server, "powerhub"),
                ObjectType::Active(ActiveObjectType::Attacker) => ItemLoader::from_name(server, "attacker"),
                ObjectType::Inactive(InactiveObjectType::Tree) => ItemLoader::from_name(server, "tree"),
            },
        }
    }

    fn into_cache(self, footprints: &Assets<Footprint>) -> InnerCache {
        InnerCache {
            objects: self
                .objects
                .map(|_, loader| loader.into_cache_item(footprints)),
        }
    }

    fn advance(&self, server: &AssetServer) -> Progress {
        self.objects
            .values()
            .map(|i| i.advance(server))
            .reduce(|a, b| a + b)
            .unwrap()
    }
}

pub(crate) struct ItemLoader {
    scene: Handle<Scene>,
    footprint: Handle<Footprint>,
}

impl ItemLoader {
    fn from_name(server: &AssetServer, name: &str) -> Self {
        let mut model_path = PathBuf::new();
        model_path.push("models");
        model_path.push(format!("{}.glb", name));

        let mut footprint_path = PathBuf::new();
        footprint_path.push("objects");
        footprint_path.push(format!("{}.obj.json", name));

        Self {
            scene: server.load(AssetPath::new(model_path, Some("Scene0".to_owned()))),
            footprint: server.load(footprint_path),
        }
    }

    fn into_cache_item(self, footprints: &Assets<Footprint>) -> CacheItem {
        let footprint = footprints.get(&self.footprint).unwrap();
        CacheItem {
            scene: self.scene,
            ichnography: Ichnography::from(footprint),
        }
    }

    fn advance(&self, server: &AssetServer) -> Progress {
        Self::advance_single(server, &self.scene) + Self::advance_single(server, &self.footprint)
    }

    fn advance_single<T: Asset>(server: &AssetServer, handle: &Handle<T>) -> Progress {
        match server.get_load_state(handle) {
            LoadState::Failed => panic!("Cache item loading failed"),
            LoadState::Unloaded => panic!("Cache item is unexpectedly unloaded"),
            LoadState::NotLoaded => false.into(),
            LoadState::Loading => false.into(),
            LoadState::Loaded => true.into(),
        }
    }
}

fn setup(mut commands: Commands, server: Res<AssetServer>) {
    commands.insert_resource(CacheLoader::load(server.as_ref()));
}

fn check_status(
    mut commands: Commands,
    mut progress: Local<Progress>,
    // keep it boxed so the memory can be freed (the system stays around forever)
    mut loader: Local<Option<Box<CacheLoader>>>,
    cache: Option<Res<ObjectCache>>,
    server: Res<AssetServer>,
    footprints: Res<Assets<Footprint>>,
) -> Progress {
    if cache.is_some() {
        debug_assert!(loader.is_none());
    } else if loader.is_none() && cache.is_none() {
        *loader = Some(Box::new(CacheLoader::load(server.as_ref())));
    } else {
        *progress = loader.as_ref().unwrap().advance(server.as_ref());
        if progress.done >= progress.total {
            let mut ready_loader = None;
            std::mem::swap(&mut ready_loader, &mut loader);
            let inner_cache = ready_loader.unwrap().into_cache(footprints.as_ref());
            commands.insert_resource(ObjectCache::new(inner_cache));
        }
    }

    *progress
}