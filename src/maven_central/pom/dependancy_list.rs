use parking_lot::RwLock;
use reqwest::Client;
use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    sync::{Arc, LazyLock},
};

use log::{debug, trace};
use maven_version::Maven3ArtifactVersion;
use regex::{Regex, RegexBuilder};
use tokio::{
    runtime::Runtime,
    spawn,
    sync::Notify,
    task::{JoinHandle, JoinSet},
};

use crate::maven_central::{
    MavenError, MavenId, MavenIdBuf,
    fetch_async::fetch_pom,
    pom::{
        dependancy_list_structs::{
            Cache, Dependancy, DependancyList, MavenDependancy, MavenDependancyList, PomState,
        },
        pom::{DependancyType, MavenPom, Scope},
    },
};

#[derive(Clone)]
struct ResolveContext {
    cache: Cache,
    list: DependancyList,
    client: Client,
}

impl MavenDependancyList {
    async fn runtime_entry(
        id: MavenIdBuf,
        scope: Option<Scope>,
    ) -> Result<Vec<MavenDependancy>, MavenError> {
        log::info!("Creating POM list for {}", id);

        let cache = Arc::new(RwLock::new(HashMap::new()));
        let dep_list = Arc::new(RwLock::new(Vec::new()));
        let client = Client::new();

        let ctx: ResolveContext = ResolveContext {
            cache,
            list: dep_list.clone(),
            client,
        };
        Self::resolve_pom(id.clone(), ctx, scope).await?;

        let mut map: HashMap<u64, MavenDependancy> = HashMap::new();

        let dep_list = Arc::into_inner(dep_list)
            .expect("Runtime completed Arc released")
            .into_inner();

        // takes the newest version
        for dep in dep_list.into_iter() {
            let hash = Self::hash_maven_bom_id(&dep.id.group, &dep.id.artifact);
            if let Some(lookup) = map.get(&hash) {
                let lookup_version = Maven3ArtifactVersion::new(&lookup.id.version);
                let new_version = Maven3ArtifactVersion::new(&dep.id.version);

                if new_version > lookup_version {
                    map.insert(hash, dep);
                }
            } else {
                map.insert(hash, dep);
            }
        }
        let mut list: Vec<MavenDependancy> = map.into_values().collect();

        // set the root dependancy
        for dep in list.iter_mut() {
            if dep.id == id {
                dep.root = true;
            }
        }
        Ok(list)
    }
    pub fn new(id: MavenIdBuf, scope: Option<Scope>) -> Result<Vec<MavenDependancy>, MavenError> {
        let rt = Runtime::new().unwrap();

        rt.block_on(Self::runtime_entry(id, scope))
    }
    async fn resolve_pom(
        id: MavenIdBuf,
        ctx: ResolveContext,
        scope: Option<Scope>,
    ) -> Result<Arc<MavenPom>, MavenError> {
        log::debug!("Resolving POM for {}", id);
        let hash = Self::hash_maven_id(&id.as_maven_id());

        // ISSUE: Its not send if i do it this way, idk, it has to be done this kind of ugly
        // way to make the compiler happy
        let mut waiting: Option<Arc<Notify>> = None;
        {
            let read_cache = ctx.cache.read();

            if let Some(pom) = read_cache.get(&hash) {
                log::debug!("Cache hit POM with hash: {} -> {}", hash, id);
                // ISSUE: i would have a match here and just wait in here but compiler does not like that
                if let PomState::Resolving(n) = pom {
                    waiting = Some(n.clone());
                }
                if let PomState::Resolved(pom) = pom {
                    return Ok(Arc::clone(pom));
                }
            }
        }

        if let Some(wait) = waiting {
            wait.notified().await;
            let cache = ctx.cache.read();
            match cache.get(&hash).unwrap() {
                PomState::Resolved(pom) => return Ok(Arc::clone(pom)),
                _ => unreachable!("Notified but cache not updated, bug"),
            }
        }

        let mut pom = fetch_pom(ctx.client.clone(), &id.as_maven_id()).await?;

        if let DependancyType::Other(other_packaging) = &pom.packaging {
            log::error!(
                "Found unknown packaging type \"{}\" on {}:{}:{}",
                other_packaging,
                pom.group_id,
                pom.artifact_id,
                pom.version
            );
        }
        let notify = Arc::new(Notify::new());

        log::debug!("(Cache Miss) Fetched POM with hash: {} -> {}", hash, id);
        {
            let mut write_cache = ctx.cache.write();
            write_cache.insert(hash, PomState::Resolving(notify.clone()));
        }

        Self::resolve_properties_inital(&mut pom);

        let parent_handle = Self::parent_handle(&pom, &ctx);

        if let Some(parent_handle) = parent_handle {
            let parent_result = parent_handle.await?;
            let parent_pom = parent_result?;
            let mut parent_props = parent_pom.properties.map.clone();
            parent_props.extend(pom.properties.map);

            pom.properties.map = parent_props;

            // backwords for right now
            pom.dependency_management_map
                .extend(parent_pom.dependency_management_map.clone());

            Self::resolve_properties_inital(&mut pom);
        }

        let mut dependancy_list: Vec<Dependancy> = Vec::new();

        if scope != Some(Scope::Provided) {
            let bom_handles = Self::bom_handles(&pom, &ctx);
            if let Some(mut bom_handles) = bom_handles {
                while let Some(result) = bom_handles.join_next().await {
                    let bom_pom = result??;
                    log::debug!(
                        "Found BOM import: {}:{}:{} for {}",
                        bom_pom.group_id,
                        bom_pom.artifact_id,
                        bom_pom.version,
                        id
                    );
                    // extend properties
                    let mut bom_props = bom_pom.properties.map.clone();
                    bom_props.extend(pom.properties.map);
                    pom.properties.map = bom_props;

                    // backwords
                    pom.dependency_management_map
                        .extend(bom_pom.dependency_management_map.clone());
                }
            }
            Self::resolve_properties_inital(&mut pom);

            // tracks the dep list for the dependancy list

            let dependacy_handles = Self::dependancy_handles(&pom, &ctx);
            if let Some(mut dependacy_handles) = dependacy_handles {
                while let Some(result) = dependacy_handles.join_next().await {
                    let (dep_result, id) = result?;
                    let dep_pom = dep_result?;
                    let mut dep_props = dep_pom.properties.map.clone();
                    dep_props.extend(pom.properties.map);
                    pom.properties.map = dep_props;

                    dependancy_list.push(Dependancy { id });
                }
            }
        }

        Self::resolve_properties_final(&mut pom);

        if pom.packaging != DependancyType::Pom
            && !matches!(pom.packaging, DependancyType::Other(_))
        {
            let mut write_list = ctx.list.write();
            write_list.push(MavenDependancy {
                id: MavenIdBuf::new(id.group, id.artifact, id.version),
                dependancy_type: pom.packaging.clone(),
                dependancies: dependancy_list,
                root: false,
                scope: Scope::Compile,
            });
        }

        let arc_pom = Arc::new(pom);
        let arc_pom_clone = arc_pom.clone();
        {
            {
                let mut write_cache = ctx.cache.write();
                write_cache.insert(hash, PomState::Resolved(arc_pom));
            }
            notify.notify_waiters();
        }

        Ok(arc_pom_clone)
    }
    pub fn hash_maven_id(id: &MavenId) -> u64 {
        let mut hasher = DefaultHasher::new();
        id.hash(&mut hasher);

        hasher.finish()
    }
    pub fn hash_maven_bom_id(group: &str, artifact: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        (group, artifact).hash(&mut hasher);

        hasher.finish()
    }
    pub fn resolve_properties_inital(pom: &mut MavenPom) {
        Self::resolve_properties(pom, resolve_string);
    }
    pub fn resolve_properties_final(pom: &mut MavenPom) {
        Self::resolve_properties(pom, resolve_string_final);
    }
    fn resolve_properties(
        pom: &mut MavenPom,
        mut resolver: impl FnMut(&mut String, &HashMap<String, String>),
    ) {
        debug!(
            "Resolving properties for {}:{}:{}",
            pom.group_id, pom.artifact_id, pom.version,
        );
        let props = &mut pom.properties;
        // resolve the properties of the properties
        let c = props.map.clone();
        for map_prop in props.map.values_mut() {
            resolver(map_prop, &c);
        }

        resolver(&mut pom.version, &props.map);
        resolver(&mut pom.group_id, &props.map);

        // Resolve properties in dependency management
        if let Some(ref mut dep_mgmt) = pom.dependency_management {
            for dep in &mut dep_mgmt.dependencies.dependency {
                if let Some(ref mut version) = dep.version {
                    resolver(version, &props.map);
                }
            }
        }

        // Resolve properties in dependencies
        if let Some(ref mut deps) = pom.dependencies {
            for dep in &mut deps.dependency {
                if let Some(ref mut version) = dep.version {
                    resolver(version, &props.map);
                }
                resolver(&mut dep.group_id, &props.map);
            }
        }

        //resolve the dependency_management_map
        for map_value in pom.dependency_management_map.values_mut() {
            resolver(map_value, &props.map);
        }
    }
    fn parent_handle(
        pom: &MavenPom,
        ctx: &ResolveContext,
    ) -> Option<JoinHandle<Result<Arc<MavenPom>, MavenError>>> {
        if let Some(parent) = &pom.parent {
            log::debug!(
                "Found parent POM: {}:{}:{}",
                parent.group_id,
                parent.artifact_id,
                parent.version,
            );
            let ctx_clone = ctx.clone();
            let owned_id: MavenIdBuf =
                MavenIdBuf::new(&parent.group_id, &parent.artifact_id, &parent.version);

            let handle = spawn(async move { Self::resolve_pom(owned_id, ctx_clone, None).await });
            return Some(handle);
        }
        None
    }
    fn bom_handles(
        pom: &MavenPom,
        ctx: &ResolveContext,
    ) -> Option<JoinSet<Result<Arc<MavenPom>, MavenError>>> {
        if let Some(dep_management) = &pom.dependency_management {
            let mut set: JoinSet<Result<Arc<MavenPom>, MavenError>> = JoinSet::new();
            for dep in &dep_management.dependencies.dependency {
                let scope = dep.scope;
                if scope == Scope::Import && !dep.optional {
                    let bom_version = dep.version.as_ref().expect("Bom is missing version");
                    let id = MavenId::new(&dep.group_id, &dep.artifact_id, bom_version);

                    let ctx_clone = ctx.clone();
                    let owned_id: MavenIdBuf = id.into();
                    set.spawn(
                        async move { Self::resolve_pom(owned_id, ctx_clone, Some(scope)).await },
                    );
                }
            }
            return Some(set);
        }
        None
    }
    fn dependancy_handles(
        pom: &MavenPom,
        ctx: &ResolveContext,
    ) -> Option<JoinSet<(Result<Arc<MavenPom>, MavenError>, MavenIdBuf)>> {
        if let Some(deps) = &pom.dependencies {
            let mut set = JoinSet::new();
            for dep in &deps.dependency {
                let scope = &dep.scope;
                if ![Scope::Compile, Scope::Runtime, Scope::Provided].contains(scope)
                    || dep.optional
                {
                    continue;
                }

                let dep_version = dep.version.as_ref().unwrap_or_else(|| {
                    let dep_bom_hash = Self::hash_maven_bom_id(&dep.group_id, &dep.artifact_id);
                    let found_version = pom.dependency_management_map.get(&dep_bom_hash);

                    debug!(
                        "Found versioning in Bom for {}:{}, version: {}",
                        &dep.group_id,
                        &dep.artifact_id,
                        found_version.unwrap_or(&"(Blank)".to_string())
                    );
                    found_version.unwrap_or_else(|| {
                        panic!("unable to find bom version");
                        // panic!("Expected to find version in bom list, pom: {}, hash: {}. bom list: {:#?}",
                        // id, hash, pom.dependency_management_map)
                    })
                });
                let ctx_clone = ctx.clone();
                let owned_id: MavenIdBuf =
                    MavenIdBuf::new(dep.group_id.clone(), dep.artifact_id.clone(), dep_version);

                let scope = *scope;
                set.spawn(async move {
                    (
                        Self::resolve_pom(owned_id.clone(), ctx_clone, Some(scope)).await,
                        owned_id,
                    )
                });
            }
            return Some(set);
        }
        None
    }
}
static PROPERTY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"\$\{(?<properties>\S+)\}")
        .swap_greed(true)
        .build()
        .expect("Property Regex is not valid")
});
fn resolve_string(label: &mut String, map: &HashMap<String, String>) {
    let mut replaced = label.to_string();
    for matches in PROPERTY_REGEX.captures_iter(label) {
        let name = matches.name("properties");
        if let Some(capture) = name
            && let Some(property) = map.get(capture.as_str())
        {
            replaced = replaced.replace(&format!("${{{}}}", capture.as_str()), property);
        }
    }

    *label = replaced;
}
fn resolve_string_final(label: &mut String, map: &HashMap<String, String>) {
    let mut replaced = label.to_string();
    for matches in PROPERTY_REGEX.captures_iter(label) {
        let name = matches.name("properties");
        if let Some(capture) = name {
            if let Some(property) = map.get(capture.as_str()) {
                replaced = replaced.replace(&format!("${{{}}}", capture.as_str()), property);
            } else {
                trace!(
                    "Property found in field but not present in map, property: {}",
                    capture.as_str()
                );
            }
        }
    }

    *label = replaced;
}
