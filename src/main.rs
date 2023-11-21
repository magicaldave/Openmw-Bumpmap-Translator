use std::collections::HashSet;
use std::fs::{rename, File};
use std::io::Write;
use std::path::PathBuf;

use openmw_cfg::{get_config, get_data_dirs};
use tes3::nif::*;
use walkdir::WalkDir;

fn main() -> std::io::Result<()> {
    let incompatible_effects = vec!["arefl_001.dds", "arefl_014.dds"];
    let mut log = Vec::new();
    let mesh_collection = collect_meshes(&mut log);
    for (mut mesh, mesh_path) in mesh_collection {
        let mut remove = HashSet::new();

        // collect textures to be removed
        for (link, texture) in mesh.objects_of_type_with_link::<NiSourceTexture>() {
            if let TextureSource::External(file_name) = &texture.source {
                let tex = file_name.to_ascii_lowercase();
                if tex.ends_with("_nm.dds") || incompatible_effects.contains(&tex.as_str()) {
                    remove.insert(link.key);
                }
            }
        }

        // unlink removed textures from texturing properties
        for (link, property) in mesh.objects_of_type_mut_with_link::<NiTexturingProperty>() {
            for texture_map in &mut property.texture_maps {
                let texture = match texture_map {
                    Some(TextureMap::Map(map)) => map.texture,
                    Some(TextureMap::BumpMap(map)) => map.texture,
                    _ => continue,
                };
                if remove.contains(&texture.key) {
                    *texture_map = None;
                }
            }
            // if it has no texture maps left we'll remove it too
            if property.texture_maps.iter().all(Option::is_none) {
                remove.insert(link.key);
            }
        }

        // unlink removed texturing properties from objects
        for object in mesh.objects_of_type_mut::<NiAVObject>() {
            object.properties.retain(|link| !remove.contains(&link.key));
        }

        for (link, property) in mesh.objects_of_type_mut_with_link::<NiTextureEffect>() {
            if remove.contains(&property.source_texture.key) {
                remove.insert(link.key);
                let _ = write!(
                    &mut log,
                    "Removed source texture node: {:?}\n",
                    property.source_texture
                )?;
            }
        }

        // unlink removed texturing properties from objects
        for object in mesh.objects_of_type_mut::<NiNode>() {
            let log_node = object.name.clone();
            object.effects.retain(|link| {
                if remove.contains(&link.key) {
                    let _ = write!(
                        &mut log,
                        "Removed effect node: {:?} from NiNode: {}\n",
                        &link.key, log_node
                    );
                    return false;
                } else {
                    return true;
                }
            });
            object.children.retain(|link| {
                if remove.contains(&link.key) {
                    let _ = write!(
                        &mut log,
                        "Removed child node: {:?} from NiNode: {}\n",
                        &link.key, log_node
                    );
                    return false;
                } else {
                    return true;
                }
            });
        }

        if remove.len() > 0 {
            // then finally remove all the stuff
            for key in &remove {
                mesh.objects.remove(*key);
            }

            let _ = write!(&mut log, "Removed Nodes: {:?}\n", remove)?;

            mesh.save_path(format!("{}", mesh_path.display()))
                .expect("Mesh writing failed! Something bad happened.");

            let _ = write!(&mut log, "{} was patched\n", mesh_path.display())?;
        }
    }

    let mut log_file = File::create("bumpmap-translator.log")?;
    let _ = log_file.write_all(log.as_slice());

    Ok(())
}

fn collect_meshes(log: &mut Vec<u8>) -> Vec<(NiStream, PathBuf)> {
    let dirs = get_data_dirs(&get_config().unwrap()).unwrap();
    let mut meshes: Vec<(NiStream, PathBuf)> = vec![];
    for dir in dirs {
        // let affected_meshes: Vec<(NiStream, PathBuf)> = vec![];
        let recursive_directory = WalkDir::new(dir).into_iter().filter_map(|e| e.ok());
        for entry in recursive_directory {
            let path = entry.path();
            match path.extension() {
                None => continue,
                Some(os_str) => match os_str.to_str() {
                    Some("dds") => {
                        if let Some(texture) = entry.file_name().to_str() {
                            if !texture.ends_with("_nm.dds") {
                                continue;
                            };
                            let _ = rename(path, path.with_file_name(texture.replace("_nm", "_n")));
                            let _ = write!(log, "Renamed bumpMap texture: {:?}\n", &path);
                        }
                    }
                    Some("nif") => {
                        if let Ok(nif) = NiStream::from_path(&entry.clone().into_path()) {
                            meshes.insert(meshes.len(), (nif, entry.into_path()));
                        } else {
                            let _ = write!(log, "Failed to open: {:?}\n", &path);
                        }
                    }
                    _ => continue,
                },
            };
        }
    }
    meshes
}
