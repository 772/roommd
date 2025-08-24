use bevy::prelude::*;
use std::collections::HashMap;

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

pub struct WindowResizePlugin;

impl Plugin for WindowResizePlugin {
    #[cfg(target_arch = "wasm32")]
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_browser_resize);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn build(&self, _app: &mut App) {}
}

#[cfg(target_arch = "wasm32")]
fn handle_browser_resize(
    mut primary_query: bevy::ecs::system::Query<
        &mut bevy::window::Window,
        bevy::ecs::query::With<bevy::window::PrimaryWindow>,
    >,
) {
    let Some(wasm_window) = web_sys::window() else {
        return;
    };
    let Ok(inner_width) = wasm_window.inner_width() else {
        return;
    };
    let Ok(inner_height) = wasm_window.inner_height() else {
        return;
    };
    let Some(target_width) = inner_width.as_f64() else {
        return;
    };
    let Some(target_height) = inner_height.as_f64() else {
        return;
    };
    for mut window in &mut primary_query {
        if window.resolution.width() != (target_width as f32)
            || window.resolution.height() != (target_height as f32)
        {
            window
                .resolution
                .set(target_width as f32, target_height as f32);
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn get_input() -> String {
    use web_sys::wasm_bindgen::JsCast;
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
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Error: Exactly one markdown file (.md) must be provided.");
        std::process::exit(1);
    }
    let path = std::path::Path::new(&args[1]);
    if path.extension().map(|ext| ext != "md").unwrap_or(true) {
        eprintln!("Error: Failed to read file '{}'", path.display());
        std::process::exit(1);
    }
    std::fs::read_to_string(path).expect("Error reading file.")
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
                    canvas: Some("#mygame-canvas".into()),
                    ..default()
                }),
                ..default()
            }),
            MeshPickingPlugin,
        ))
        .add_systems(Startup, setup)
        .add_plugins(bevy_panorbit_camera::PanOrbitCameraPlugin)
        .add_plugins(WindowResizePlugin)
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
    x: i32,
    y: i32,
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
                    let pos = Position {
                        x: obj.0 as i32,
                        y: obj.1 as i32,
                    };
                    let locations = object_list.entry(obj.2).or_default();
                    match wall_name {
                        "top" => locations.top.entry(room_index).or_default().push(pos),
                        "floor" => locations.floor.entry(room_index).or_default().push(pos),
                        "back" => locations.back.entry(room_index).or_default().push(pos),
                        "right" => locations.right.entry(room_index).or_default().push(pos),
                        "left" => locations.left.entry(room_index).or_default().push(pos),
                        "front" => locations.front.entry(room_index).or_default().push(pos),
                        _ => panic!("Invalid wall_name: {wall_name}"),
                    }
                }
            }
            room_index += 1;
        }
    }

    // Step 2: Rearange the positions of the rooms.
    for _ in 0..2 {
        for locations in object_list.values() {
            let walls = [
                (&locations.left, &locations.right, 1.0, 0.0, 0.0),
                (&locations.right, &locations.left, -1.0, 0.0, 0.0),
                (&locations.top, &locations.floor, 0.0, -1.0, 0.0),
                (&locations.floor, &locations.top, 0.0, 1.0, 0.0),
                (&locations.front, &locations.back, 0.0, 0.0, -1.0),
                (&locations.back, &locations.front, 0.0, 0.0, 1.0),
            ];
            for (this_wall, other_wall, x_mul, y_mul, z_mul) in walls {
                for (current_room_index, positions) in this_wall {
                    if *current_room_index != 0 {
                        for (other_room_index, positions2) in other_wall {
                            if *other_room_index < *current_room_index {
                                let mut positions_this = positions.clone();
                                positions_this.sort_by(|a, b| a.x.cmp(&b.x).then(a.y.cmp(&b.y)));
                                let mut other_positions_mirrored = positions2.clone();

                                // Mirroring other position.
                                for position in &mut other_positions_mirrored {
                                    if y_mul == 0.0 {
                                        position.x = (rooms[*other_room_index].depth as i32
                                            * (x_mul as i32).abs()
                                            + rooms[*other_room_index].width as i32
                                                * (z_mul as i32).abs())
                                            - position.x
                                            - 1;
                                    }
                                    if y_mul != 0.0 {
                                        position.y = (rooms[*other_room_index].height as i32
                                            * (y_mul as i32).abs())
                                            - position.y
                                            - 1;
                                    }
                                }
                                other_positions_mirrored
                                    .sort_by(|a, b| a.x.cmp(&b.x).then(a.y.cmp(&b.y)));

                                // Normalize this position.
                                let x_offset_this = positions_this[0].x;
                                let y_offset_this = positions_this[0].y;
                                for position in &mut positions_this {
                                    position.x -= x_offset_this;
                                    position.y -= y_offset_this;
                                }

                                // Normalize other position.
                                let x_offset_other = other_positions_mirrored[0].x;
                                let y_offset_other = other_positions_mirrored[0].y;
                                for position in &mut other_positions_mirrored {
                                    position.x -= x_offset_other;
                                    position.y -= y_offset_other;
                                }

                                let x_full_offset = x_offset_this as f32 - x_offset_other as f32;
                                let y_full_offset = y_offset_this as f32 - y_offset_other as f32;
                                if positions_this == other_positions_mirrored {
                                    let x_room_off = rooms[*current_room_index].width / 2.0
                                        + rooms[*other_room_index].width / 2.0;
                                    let y_room_off = rooms[*current_room_index].height / 2.0
                                        + rooms[*other_room_index].height / 2.0;
                                    let z_room_off = rooms[*current_room_index].depth / 2.0
                                        + rooms[*other_room_index].depth / 2.0;
                                    rooms[*current_room_index].x =
                                        rooms[*other_room_index].x + x_room_off * x_mul;
                                    rooms[*current_room_index].y =
                                        rooms[*other_room_index].y + y_room_off * y_mul;
                                    rooms[*current_room_index].z =
                                        rooms[*other_room_index].z + z_room_off * z_mul;
                                    // Left, Right.
                                    if x_mul != 0.0 {
                                        rooms[*current_room_index].z += x_full_offset;
                                        rooms[*current_room_index].y += y_full_offset;
                                    }
                                    // Top, Bottom.
                                    if y_mul != 0.0 {
                                        rooms[*current_room_index].z += y_full_offset;
                                        rooms[*current_room_index].x -= x_full_offset;
                                    }
                                    // Front, Back.
                                    if z_mul != 0.0 {
                                        println!(
                                            "Apply front/back offset {x_full_offset} {y_full_offset}."
                                        );
                                        rooms[*current_room_index].x += x_full_offset;
                                        rooms[*current_room_index].y += y_full_offset;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Step 3: Spawn rooms and objects.
    let scaling2 = 1.0 / 18.0;
    let scaling = scaling2 * 0.999;
    for room in rooms {
        commands
            .spawn((
                Mesh3d(
                    meshes.add(
                        Mesh::from(Cuboid::new(
                            room.width * scaling2,
                            room.height * scaling2,
                            room.depth * scaling2,
                        ))
                        .with_generated_tangents()
                        .unwrap(),
                    ),
                ),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgba(1.0, 1.0, 1.0, 1.0),
                    emissive: Color::srgba(0.0, 0.0, 0.0, 1.0).into(),
                    cull_mode: Some(bevy::render::render_resource::Face::Front),
                    double_sided: true,
                    unlit: false,
                    fog_enabled: true,
                    ..default()
                })),
                Pickable::IGNORE,
                bevy::pbr::NotShadowCaster,
                Transform::from_translation(Vec3::new(
                    room.x * scaling2,
                    room.y * scaling2,
                    room.z * scaling2,
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
                                scaling * (obj.0 as f32 - room.width / 2.0 + 0.5),
                                scaling * (room.height / 2.0 - 0.1),
                                scaling * (0.0 - obj.1 as f32 + room.depth / 2.0 - 0.5),
                            )),
                            Object(obj.2),
                            bevy::pbr::NotShadowCaster,
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
                                scaling * (obj.0 as f32 - room.width / 2.0 + 0.5),
                                scaling * (0.0 - obj.1 as f32 + room.height / 2.0 - 0.5),
                                scaling * (0.0 - room.depth / 2.0 - 0.1),
                            )),
                            Object(obj.2),
                            bevy::pbr::NotShadowCaster,
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
                                scaling * (room.width / 2.0 - 0.1),
                                scaling * (0.0 - obj.1 as f32 + room.depth / 2.0 - 0.5),
                                scaling * (obj.0 as f32 - room.height / 2.0 + 0.5),
                            )),
                            Object(obj.2),
                            bevy::pbr::NotShadowCaster,
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
                                scaling * (0.0 - obj.0 as f32 + room.width / 2.0 - 0.5),
                                scaling * (0.0 - obj.1 as f32 + room.height / 2.0 - 0.5),
                                scaling * (room.depth / 2.0 + 0.1),
                            )),
                            Object(obj.2),
                            bevy::pbr::NotShadowCaster,
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
                                scaling * (0.0 - room.width / 2.0 + 0.1),
                                scaling * (0.0 - obj.1 as f32 + room.depth / 2.0 - 0.5),
                                scaling * (0.0 - obj.0 as f32 + room.height / 2.0 - 0.5),
                            )),
                            Object(obj.2),
                            bevy::pbr::NotShadowCaster,
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
                                scaling * (obj.0 as f32 - room.width / 2.0 + 0.5),
                                scaling * (0.0 - room.height / 2.0 + 0.1),
                                scaling * (obj.1 as f32 - room.depth / 2.0 + 0.5),
                            )),
                            Object(obj.2),
                            bevy::pbr::NotShadowCaster,
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
    commands.insert_resource(AmbientLight {
        brightness: 800.,
        ..default()
    });
    commands.spawn((
        PointLight {
            intensity: 1_000_000.0,
            range: 100.,
            ..default()
        },
        Transform::from_xyz(5.0, -10.0, 2.5),
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
