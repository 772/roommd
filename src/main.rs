use bevy::prelude::*;
use std::collections::HashMap;
use web_sys::wasm_bindgen::JsCast;

#[derive(Component)]
struct Object(char);

#[derive(Resource)]
struct Descriptions(HashMap<char, String>);

#[derive(Default, Debug)]
struct LocationsOfChar {
    top: HashMap<usize, Vec<Position>>,
    floor: HashMap<usize, Vec<Position>>,
    back: HashMap<usize, Vec<Position>>,
    right: HashMap<usize, Vec<Position>>,
    left: HashMap<usize, Vec<Position>>,
    front: HashMap<usize, Vec<Position>>,
}

#[cfg(target_arch = "wasm32")]
fn get_input() -> String {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    document
        .get_element_by_id("input")
        .unwrap()
        .dyn_into::<web_sys::HtmlTextAreaElement>()
        .unwrap()
        .value()
}

#[cfg(not(target_arch = "wasm32"))]
fn get_input() -> String {
    std::fs::read_to_string("example.md").expect("Unable to read file")
}

pub fn get_letters_in_ascii_grid(
    image: Vec<&str>,
    x: usize,
    y: usize,
    wdt: usize,
    hgt: usize,
) -> Vec<(usize, usize, char)> {
    const IGNORED_CHARS: [char; 4] = ['+', '-', ' ', '|'];

    image[y..y + hgt]
        .iter()
        .map(|row| &row[x..x + wdt])
        .enumerate()
        .flat_map(|(y, row)| {
            row.chars()
                .enumerate()
                .filter(|(_, ch)| !IGNORED_CHARS.contains(ch))
                .map(move |(x, ch)| (x, y, ch))
        })
        .collect()
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: bevy::window::WindowResolution::new(1000.0, 900.0)
                        .with_scale_factor_override(1.0),
                    ..default()
                }),
                ..default()
            }),
            MeshPickingPlugin,
        ))
        .add_systems(Startup, setup)
        .add_plugins(bevy_panorbit_camera::PanOrbitCameraPlugin)
        .insert_resource(Descriptions(HashMap::new()))
        .run();
}

struct Room {
    x: f32,
    y: f32,
    z: f32,
    width: f32,
    height: f32,
    depth: f32,
    top: Vec<(usize, usize, char)>,
    floor: Vec<(usize, usize, char)>,
    left: Vec<(usize, usize, char)>,
    right: Vec<(usize, usize, char)>,
    front: Vec<(usize, usize, char)>,
    back: Vec<(usize, usize, char)>,
}

#[derive(Debug, PartialEq, Clone)]
struct Position {
    x: usize,
    y: usize,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut descriptions: ResMut<Descriptions>,
) {
    // Step 1: Read markdown and create lists ob objects.
    let text = get_input();
    let mut object_list: HashMap<char, LocationsOfChar> = HashMap::new();
    let mut room_index = 0;
    let mut rooms: Vec<Room> = Vec::new();
    for section in text.split('#').filter(|s| !s.trim().is_empty()) {
        let mut lines = section.trim().lines();
        let first_line = lines.next().unwrap_or("").trim();
        let name = first_line.to_string();
        let content = lines.collect::<Vec<&str>>().join("\n").trim().to_string();
        if name.len() == 1 {
            let name = name.chars().next().unwrap();
            let desc = content;
            descriptions.0.insert(name, desc);
        } else {
            let lines: Vec<&str> = content.trim().lines().collect();
            let first_line = lines[0].trim();
            let width: usize = first_line.len();
            let depth: usize = 1 + lines
                .clone()
                .into_iter()
                .enumerate()
                .skip(1)
                .find_map(|(i, line)| line.starts_with('+').then_some(i))
                .unwrap();
            let line_count = lines.len();
            let height = line_count - 2 * depth;
            assert_eq!(
                2 * (width * depth + width * height + depth * height),
                content.replace(['\n', '\r'], "").len()
            );
            let (top, back, right, front, left, floor) = (
                get_letters_in_ascii_grid(lines.clone(), 0, 0, width, depth),
                get_letters_in_ascii_grid(lines.clone(), 0, depth, width, height),
                get_letters_in_ascii_grid(lines.clone(), width, depth, depth, height),
                get_letters_in_ascii_grid(lines.clone(), width + depth, depth, width, height),
                get_letters_in_ascii_grid(lines.clone(), 2 * width + depth, depth, depth, height),
                get_letters_in_ascii_grid(lines.clone(), 0, depth + height, width, depth),
            );
            rooms.push(Room {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                width: width as f32,
                height: height as f32,
                depth: depth as f32,
                top: top.clone(),
                floor: floor.clone(),
                left: left.clone(),
                right: right.clone(),
                front: front.clone(),
                back: back.clone(),
            });
            let room = rooms.last().unwrap();
            let walls = [
                (&room.top, "top"),
                (&room.back, "back"),
                (&room.right, "right"),
                (&room.front, "front"),
                (&room.left, "left"),
                (&room.floor, "floor"),
            ];
            for (objects, wall_name) in walls {
                for obj in objects {
                    let pos = Position { x: obj.0, y: obj.1 };
                    let locations = object_list.entry(obj.2).or_default();
                    match wall_name {
                        "top" => locations.top.entry(room_index).or_default().push(pos),
                        "floor" => locations.floor.entry(room_index).or_default().push(pos),
                        "back" => locations.back.entry(room_index).or_default().push(pos),
                        "right" => locations.right.entry(room_index).or_default().push(pos),
                        "left" => locations.left.entry(room_index).or_default().push(pos),
                        "front" => locations.front.entry(room_index).or_default().push(pos),
                        _ => panic!("Invalid wall_name: {}", wall_name),
                    }
                }
            }
            room_index += 1;
        }
    }

    // Step 2: Rearange the positions of the rooms.
    for _ in 0..2 {
        for (_character, locations) in &object_list {
            let mut loop_order = 1;
            for (current_room_index, positions) in &locations.left {
                if loop_order < 1 {
                    loop_order = 2
                };
                if *current_room_index != 0 {
                    for (other_room_index, positions2) in &locations.right {
                        if *other_room_index < *current_room_index {
                            let mut other_positions_mirrored = positions2.clone();
                            for i in 0..other_positions_mirrored.len() {
                                other_positions_mirrored[i].x = rooms[*other_room_index].depth
                                    as usize
                                    - 1
                                    - other_positions_mirrored[i].x;
                            }
                            if *positions == other_positions_mirrored {
                                rooms[*current_room_index].x = rooms[*other_room_index].x
                                    + rooms[*current_room_index].width / 2.0
                                    + rooms[*other_room_index].width / 2.0;
                                rooms[*current_room_index].y = rooms[*other_room_index].y;
                                rooms[*current_room_index].z = rooms[*other_room_index].z;
                            }
                        }
                    }
                }
            }
            for (current_room_index, positions) in &locations.right {
                if loop_order < 2 {
                    loop_order = 3
                };
                if *current_room_index != 0 {
                    for (other_room_index, positions2) in &locations.left {
                        if *other_room_index < *current_room_index {
                            let mut other_positions_mirrored = positions2.clone();
                            for i in 0..other_positions_mirrored.len() {
                                other_positions_mirrored[i].x = rooms[*other_room_index].depth
                                    as usize
                                    - 1
                                    - other_positions_mirrored[i].x;
                            }
                            if *positions == other_positions_mirrored {
                                rooms[*current_room_index].x = rooms[*other_room_index].x
                                    - rooms[*current_room_index].width / 2.0
                                    - rooms[*other_room_index].width / 2.0;
                                rooms[*current_room_index].y = rooms[*other_room_index].y;
                                rooms[*current_room_index].z = rooms[*other_room_index].z;
                            }
                        }
                    }
                }
            }
            for (current_room_index, positions) in &locations.top {
                if loop_order < 3 {
                    loop_order = 4
                };
                if *current_room_index != 0 {
                    for (other_room_index, positions2) in &locations.floor {
                        if *other_room_index < *current_room_index {
                            let mut other_positions_mirrored = positions2.clone();
                            for i in 0..other_positions_mirrored.len() {
                                other_positions_mirrored[i].y = rooms[*other_room_index].height
                                    as usize
                                    - 1
                                    - other_positions_mirrored[i].y;
                            }
                            if *positions == other_positions_mirrored {
                                rooms[*current_room_index].x = rooms[*other_room_index].x;
                                rooms[*current_room_index].y = rooms[*other_room_index].y
                                    - rooms[*current_room_index].height / 2.0
                                    - rooms[*other_room_index].height / 2.0;
                                rooms[*current_room_index].z = rooms[*other_room_index].z;
                            }
                        }
                    }
                }
            }
            for (current_room_index, positions) in &locations.floor {
                if loop_order < 4 {
                    loop_order = 5
                };
                if *current_room_index != 0 {
                    for (other_room_index, positions2) in &locations.top {
                        if *other_room_index < *current_room_index {
                            let mut other_positions_mirrored = positions2.clone();
                            for i in 0..other_positions_mirrored.len() {
                                other_positions_mirrored[i].y = rooms[*other_room_index].height
                                    as usize
                                    - 1
                                    - other_positions_mirrored[i].y;
                            }
                            if *positions == other_positions_mirrored {
                                rooms[*current_room_index].x = rooms[*other_room_index].x;
                                rooms[*current_room_index].y = rooms[*other_room_index].y
                                    + rooms[*current_room_index].height / 2.0
                                    + rooms[*other_room_index].height / 2.0;
                                rooms[*current_room_index].z = rooms[*other_room_index].z;
                            }
                        }
                    }
                }
            }
            for (current_room_index, positions) in &locations.front {
                if loop_order < 5 {
                    loop_order = 6
                };
                if *current_room_index != 0 {
                    for (other_room_index, positions2) in &locations.back {
                        if *other_room_index < *current_room_index {
                            let mut other_positions_mirrored = positions2.clone();
                            for i in 0..other_positions_mirrored.len() {
                                other_positions_mirrored[i].x = rooms[*other_room_index].width
                                    as usize
                                    - 1
                                    - other_positions_mirrored[i].x;
                            }
                            if *positions == other_positions_mirrored {
                                rooms[*current_room_index].x = rooms[*other_room_index].x;
                                rooms[*current_room_index].y = rooms[*other_room_index].y;
                                rooms[*current_room_index].z = rooms[*other_room_index].z
                                    - rooms[*current_room_index].depth / 2.0
                                    - rooms[*other_room_index].depth / 2.0;
                            }
                        }
                    }
                }
            }
            for (current_room_index, positions) in &locations.back {
                if loop_order < 6 {
                    loop_order = 7
                };
                if *current_room_index != 0 {
                    for (other_room_index, positions2) in &locations.front {
                        if *other_room_index < *current_room_index {
                            let mut other_positions_mirrored = positions2.clone();
                            for i in 0..other_positions_mirrored.len() {
                                other_positions_mirrored[i].x = rooms[*other_room_index].width
                                    as usize
                                    - 1
                                    - other_positions_mirrored[i].x;
                            }
                            if *positions == other_positions_mirrored {
                                rooms[*current_room_index].x = rooms[*other_room_index].x;
                                rooms[*current_room_index].y = rooms[*other_room_index].y;
                                rooms[*current_room_index].z = rooms[*other_room_index].z
                                    + rooms[*current_room_index].depth / 2.0
                                    + rooms[*other_room_index].depth / 2.0;
                            }
                        }
                    }
                }
            }
        }
    }

    // Step 3: Spawn rooms and objects.
    let scaling = 1.0 / 18.0;
    for room in rooms {
        commands
            .spawn((
                Mesh3d(meshes.add(Cuboid::new(
                    room.width * scaling,
                    room.height * scaling,
                    room.depth * scaling,
                ))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgba(0.2, 0.3, 0.4, 0.7),
                    metallic: 0.7,
                    perceptual_roughness: 0.3,
                    reflectance: 0.8,
                    emissive: Color::srgba(0.05, 0.05, 0.07, 1.0).into(),
                    cull_mode: Some(bevy::render::render_resource::Face::Front),
                    double_sided: true,
                    unlit: false,
                    fog_enabled: true,
                    ..default()
                })),
                bevy::pbr::NotShadowCaster,
                Pickable::IGNORE,
                Transform::from_translation(Vec3::new(
                    room.x * scaling,
                    room.y * scaling,
                    room.z * scaling,
                )),
            ))
            .with_children(|parent| {
                for obj in &room.top {
                    let white_matl = materials.add(Color::Hsla(char_to_color(obj.2)));
                    parent
                        .spawn((
                            Mesh3d(meshes.add(Cuboid::new(scaling, scaling * 0.2, scaling))),
                            MeshMaterial3d(white_matl.clone()),
                            Transform::from_translation(Vec3::new(
                                scaling * (obj.0 as f32 - room.width as f32 / 2.0 + 0.5),
                                scaling * (room.height as f32 / 2.0 - 0.2),
                                scaling * (0.0 - obj.1 as f32 + room.depth as f32 / 2.0 - 0.5),
                            )),
                            Object(obj.2),
                        ))
                        .observe(update_material_on::<Pointer<Over>>(
                            materials.add(Color::Hsla(Hsla {
                                lightness: 0.8,
                                ..char_to_color(obj.2)
                            })),
                            obj.2,
                        ))
                        .observe(update_material_on::<Pointer<Out>>(
                            white_matl.clone(),
                            obj.2,
                        ));
                }
                for obj in &room.back {
                    let white_matl = materials.add(Color::Hsla(char_to_color(obj.2)));
                    parent
                        .spawn((
                            Mesh3d(meshes.add(Cuboid::new(scaling, scaling, scaling * 0.2))),
                            MeshMaterial3d(white_matl.clone()),
                            Transform::from_translation(Vec3::new(
                                scaling * (obj.0 as f32 - room.width as f32 / 2.0 + 0.5),
                                scaling * (0.0 - obj.1 as f32 + room.height as f32 / 2.0 - 0.5),
                                scaling * (0.0 - room.depth as f32 / 2.0 + 0.2),
                            )),
                            Object(obj.2),
                        ))
                        .observe(update_material_on::<Pointer<Over>>(
                            materials.add(Color::Hsla(Hsla {
                                lightness: 0.8,
                                ..char_to_color(obj.2)
                            })),
                            obj.2,
                        ))
                        .observe(update_material_on::<Pointer<Out>>(
                            white_matl.clone(),
                            obj.2,
                        ));
                }
                for obj in &room.right {
                    let white_matl = materials.add(Color::Hsla(char_to_color(obj.2)));
                    parent
                        .spawn((
                            Mesh3d(meshes.add(Cuboid::new(scaling * 0.2, scaling, scaling))),
                            MeshMaterial3d(white_matl.clone()),
                            Transform::from_translation(Vec3::new(
                                scaling * (room.width as f32 / 2.0 - 0.2),
                                scaling * (0.0 - obj.1 as f32 + room.depth as f32 / 2.0 - 0.5),
                                scaling * (obj.0 as f32 - room.height as f32 / 2.0 + 0.5),
                            )),
                            Object(obj.2),
                        ))
                        .observe(update_material_on::<Pointer<Over>>(
                            materials.add(Color::Hsla(Hsla {
                                lightness: 0.8,
                                ..char_to_color(obj.2)
                            })),
                            obj.2,
                        ))
                        .observe(update_material_on::<Pointer<Out>>(
                            white_matl.clone(),
                            obj.2,
                        ));
                }
                for obj in &room.front {
                    let white_matl = materials.add(Color::Hsla(char_to_color(obj.2)));
                    parent
                        .spawn((
                            Mesh3d(meshes.add(Cuboid::new(scaling, scaling, scaling * 0.2))),
                            MeshMaterial3d(white_matl.clone()),
                            Transform::from_translation(Vec3::new(
                                scaling * (0.0 - obj.0 as f32 + room.width as f32 / 2.0 - 0.5),
                                scaling * (0.0 - obj.1 as f32 + room.height as f32 / 2.0 - 0.5),
                                scaling * (room.depth as f32 / 2.0 - 0.2),
                            )),
                            Object(obj.2),
                        ))
                        .observe(update_material_on::<Pointer<Over>>(
                            materials.add(Color::Hsla(Hsla {
                                lightness: 0.8,
                                ..char_to_color(obj.2)
                            })),
                            obj.2,
                        ))
                        .observe(update_material_on::<Pointer<Out>>(
                            white_matl.clone(),
                            obj.2,
                        ));
                }
                for obj in &room.left {
                    let white_matl = materials.add(Color::Hsla(char_to_color(obj.2)));
                    parent
                        .spawn((
                            Mesh3d(meshes.add(Cuboid::new(scaling * 0.2, scaling, scaling))),
                            MeshMaterial3d(white_matl.clone()),
                            Transform::from_translation(Vec3::new(
                                scaling * (0.0 - room.width as f32 / 2.0 + 0.2),
                                scaling * (0.0 - obj.1 as f32 + room.depth as f32 / 2.0 - 0.5),
                                scaling * (0.0 - obj.0 as f32 + room.height as f32 / 2.0 - 0.5),
                            )),
                            Object(obj.2),
                        ))
                        .observe(update_material_on::<Pointer<Over>>(
                            materials.add(Color::Hsla(Hsla {
                                lightness: 0.8,
                                ..char_to_color(obj.2)
                            })),
                            obj.2,
                        ))
                        .observe(update_material_on::<Pointer<Out>>(
                            white_matl.clone(),
                            obj.2,
                        ));
                }
                for obj in &room.floor {
                    let white_matl = materials.add(Color::Hsla(char_to_color(obj.2)));
                    parent
                        .spawn((
                            Mesh3d(meshes.add(Cuboid::new(scaling, scaling * 0.2, scaling))),
                            MeshMaterial3d(white_matl.clone()),
                            Transform::from_translation(Vec3::new(
                                scaling * (obj.0 as f32 - room.width as f32 / 2.0 + 0.5),
                                scaling * (0.0 - room.height as f32 / 2.0 + 0.2),
                                scaling * (obj.1 as f32 - room.depth as f32 / 2.0 + 0.5),
                            )),
                            Object(obj.2),
                        ))
                        .observe(update_material_on::<Pointer<Over>>(
                            materials.add(Color::Hsla(Hsla {
                                lightness: 0.8,
                                ..char_to_color(obj.2)
                            })),
                            obj.2,
                        ))
                        .observe(update_material_on::<Pointer<Out>>(
                            white_matl.clone(),
                            obj.2,
                        ));
                }
            });
        room_index += 1;
    }

    // Spawn other stuff.
    commands.spawn(Text(
        "Hover objects to read their descriptions.".to_string(),
    ));
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(3.0, 1.0, 3.0).looking_at(Vec3::new(0.0, -0.5, 0.0), Vec3::Y),
        bevy_panorbit_camera::PanOrbitCamera::default(),
    ));
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

#[allow(clippy::type_complexity)]
fn update_material_on<E>(
    new_material: Handle<StandardMaterial>,
    ch: char,
) -> impl Fn(
    Trigger<E>,
    Query<(&mut MeshMaterial3d<StandardMaterial>, &Object)>,
    Query<&mut Text>,
    Res<Descriptions>,
) {
    move |_trigger, mut objects, mut texts, descriptions| {
        for (mut material, character) in objects.iter_mut() {
            if character.0 == ch {
                material.0 = new_material.clone();
            }
        }
        for mut text in texts.iter_mut() {
            *text = Text(
                descriptions
                    .0
                    .get(&ch)
                    .unwrap_or(&"No description available.".to_string())
                    .to_string(),
            );
        }
    }
}

pub fn char_to_color(c: char) -> Hsla {
    let hash = c as u32 * 10007; // Big prime as multiplicator.
    Hsla {
        hue: (hash % 360) as f32,
        saturation: 1.0,
        lightness: 0.5,
        alpha: 1.0,
    }
}
